#ifndef SCREENRECORDERCUSTOM_H
#define SCREENRECORDERCUSTOM_H

#include <QObject>
#include <qqml.h>
#include "ScreenCap.h"

class ScreenRecorderCustom : public QObject, public ScreenRecorder
{
    Q_OBJECT
    Q_PROPERTY(const AVFrame* frameRGB READ frameRGB CONSTANT)
private:
    const AVFrame* m_frameRGB = nullptr;
    void onFrameUpdate() override;

public:
    explicit ScreenRecorderCustom(QObject *parent = nullptr);
    const AVFrame* frameRGB();

signals:
    void frameUpdated();

public slots:
    int initCapture(int windowWidth, int windowHeight, int offsetX, int offsetY, int outputFrameWidth, int outputFrameHeight);
    int startCapture();
    int stopCapture();
};

#endif // SCREENRECORDERCUSTOM_H
