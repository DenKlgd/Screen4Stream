#include "UdpSocketCustom.h"


UdpSocketCustom::UdpSocketCustom()
{
    QObject::connect(&socket, &QUdpSocket::readyRead, this, &UdpSocketCustom::readVideoFrame);
    hosts.append(QHostAddress::LocalHost);
    avFrame = av_frame_alloc();
}

UdpSocketCustom::~UdpSocketCustom()
{
    av_frame_free(&avFrame);
}

Frame UdpSocketCustom::receivedFrame()
{
    return frame;
}

const AVFrame* UdpSocketCustom::frameRGB()
{
    avFrame->width = frame.header.width;
    avFrame->height = frame.header.height;
    avFrame->data[0] = (uint8_t*)frame.data;

    return avFrame;
}

Q_INVOKABLE bool UdpSocketCustom::openSocket()
{
    if (!socket.bind(QHostAddress::LocalHost, UdpPort))
    {
        qDebug() << "Unable to bind to port " << UdpPort;
        return false;
    }
    isSocketOpened = true;
    return true;
}

Q_INVOKABLE void UdpSocketCustom::closeSocket()
{
    socket.close();
    isSocketOpened = false;
}

Q_INVOKABLE bool UdpSocketCustom::getSocketState()
{
    return isSocketOpened;
}

// void UdpSocketCustom::sendData(const char *data, qint64 size)
// {
//     constexpr short bitCount = sizeof(qint64) * 8;

//     const qint64 datagramCount = size >> 9;
//     quint64 bytesLeft = size << (bitCount - 9);
//     bytesLeft = bytesLeft >> (bitCount - 9);

//     for (auto& host : hosts)
//     {
//         qint64 offset = 0;
//         for (qint64 i = 0; i < datagramCount; i++)
//         {
//             socket.writeDatagram(data + offset, DatagramSize, host, UdpPort);
//             offset += DatagramSize;
//         }
//         if (bytesLeft > 0)
//         {
//             socket.writeDatagram(data + offset, bytesLeft, host, UdpPort);
//         }
//     }
// }

void UdpSocketCustom::sendData(QHostAddress host, qint16 port, const char *data, qint64 size)
{
    constexpr short bitCount = sizeof(qint64) * 8;

    const qint64 datagramCount = size >> 9;
    quint64 bytesLeft = size << (bitCount - 9);
    bytesLeft = bytesLeft >> (bitCount - 9);

    qint64 offset = 0;
    for (qint64 i = 0; i < datagramCount; i++)
    {
        socket.writeDatagram(data + offset, DatagramSize, host, UdpPort);
        offset += DatagramSize;
    }
    if (bytesLeft > 0)
    {
        char dataChunk[DatagramSize];
        memcpy(dataChunk, data + offset, bytesLeft);
        socket.writeDatagram(dataChunk, DatagramSize, host, UdpPort);
    }
}

void UdpSocketCustom::sendVideoFrame(const AVFrame* frame)
{
    // constexpr int frameHeaderSize = sizeof(frameSize) + sizeof(width) + sizeof(height);
    // char frameHeader[frameHeaderSize];
    // qint32 headerOffset = 0;

    // memcpy(frameHeader, &frameSize, sizeof(frameSize));
    // headerOffset += sizeof(frameSize);

    // memcpy(frameHeader + headerOffset, &width, sizeof(width));
    // headerOffset += sizeof(width);

    // memcpy(frameHeader + headerOffset, &height, sizeof(height));
    // headerOffset += sizeof(height);

    qint64 frameSize = frame->width * frame->height * 3;

    QByteArray frameHeader;
    frameHeader.append((char*)&frameSize, sizeof(frameSize));
    frameHeader.append((char*)&frame->width, sizeof(qint32));
    frameHeader.append((char*)&frame->height, sizeof(qint32));

    for (auto& host : hosts)
    {
        //socket.writeDatagram(frameHeader, frameHeaderSize, host, UdpPort);
        socket.writeDatagram(frameHeader, host, UdpPort);
        sendData(host, UdpPort, (char*)frame->data[0], frameSize);
    }
}

void UdpSocketCustom::readVideoFrame()
{
    FrameHeader frameHeader;
    char buffer[DatagramSize];

    while (socket.hasPendingDatagrams())
    {
        if (!isHeaderReceived)
        {
            if (socket.readDatagram(buffer, DatagramSize) == frameHeaderSize)
            {
                isHeaderReceived = true;
                memcpy(&frameHeader, buffer, frameHeaderSize);

                if (frame.data == nullptr)
                {
                    frame.data = new char[frameHeader.size];
                    frame.header = frameHeader;
                }
                else if (frame.header.size != frameHeader.size)
                {
                    delete[] frame.data;
                    frame.data = new char[frameHeader.size];
                    frame.header = frameHeader;
                }
            }
        }
        else
        {
            if (socket.readDatagram(frame.data + bytesReceived, DatagramSize) == DatagramSize)
            {
                bytesReceived += DatagramSize;
                if (bytesReceived >= frame.header.size)
                {
                    bytesReceived = 0;
                    isHeaderReceived = false;
                    emit frameReceived();
                }
            }
            else
            {
                bytesReceived = 0;
                isHeaderReceived = false;
            }
        }
    }
}

