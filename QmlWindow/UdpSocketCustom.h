#ifndef UDP_SOCKET_CUSTOM_H
#define UDP_SOCKET_CUSTOM_H

#include <QObject>
#include <QList>
#include <QUdpSocket>
#include <QNetworkDatagram>
#include <QDebug>
#include "Frame.h"

extern "C"
{
    #include <libavutil/frame.h>
}

#define UdpPort 7755
#define DatagramSize 512


class UdpSocketCustom : public QObject
{
    Q_OBJECT
    Q_PROPERTY(Frame receivedFrame READ receivedFrame CONSTANT)
    Q_PROPERTY(const AVFrame* frameRGB READ frameRGB CONSTANT)
private:
    QUdpSocket socket;
    QList<QHostAddress> hosts;
    bool isSocketOpened = false;
    Frame frame;
    AVFrame* avFrame = nullptr;
    qint64 bytesReceived = 0;
    bool isHeaderReceived = false;
    static constexpr int frameHeaderSize = sizeof(FrameHeader);

public:
    UdpSocketCustom();
    ~UdpSocketCustom();
    Frame receivedFrame();
    const AVFrame* frameRGB();

private:
    void sendData(QHostAddress host, qint16 port, const char *data, qint64 size);

public:
    Q_INVOKABLE bool openSocket();
    Q_INVOKABLE void closeSocket();
    Q_INVOKABLE bool getSocketState();

signals:
    void frameReceived();

public slots:
    //void sendVideoFrame(const char* frame, qint64 frameSize, quint32 width, quint32 height);
    void sendVideoFrame(const AVFrame* frame);
    void readVideoFrame();
};

#endif