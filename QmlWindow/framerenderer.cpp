#include "framerenderer.h"
#include <QDebug>

#define screenWidth 1024
#define screenHeight 768

FrameRenderer::FrameRenderer(QQuickItem *parent) :
    QQuickPaintedItem(parent)
{
}

 void FrameRenderer::paint(QPainter *painter)
 {
    if (m_frame != nullptr)
    {
        QImage frameTexture(m_frame->data[0], m_frame->width, m_frame->height, QImage::Format::Format_RGB888);
        painter->drawImage(QRectF(0, 0, width(), height()), frameTexture);
    }
    else
    {
        painter->fillRect(0, 0, width(), height(), Qt::black);
    }
    qDebug() << "Paint";
 }

 void FrameRenderer::frameUpdated(const AVFrame* const frame)
 {
     qDebug() << "void FrameRenderer::frameUpdated()\n";

     m_frame = frame;
     update();
 }
