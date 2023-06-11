#ifndef FRAMERENDERER_H
#define FRAMERENDERER_H

#include <QImage>
#include <QBitmap>
#include <QQuickPaintedItem>
#include <QPainter>

#include <QOpenGLContext>
#include <QOpenGLFunctions>
#include "Frame.h"

extern "C"
{
    #include <libavutil/frame.h>
}


class FrameRenderer : public QQuickPaintedItem
{
    Q_OBJECT
private:
    const AVFrame* m_frame = nullptr;

protected:
    virtual void paint(QPainter *painter) Q_DECL_OVERRIDE;

public:
    FrameRenderer(QQuickItem *parent = Q_NULLPTR);

public slots:
    void frameUpdated(const AVFrame* const frame);
};

#endif // FRAMERENDERER_H
