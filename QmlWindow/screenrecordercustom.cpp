#include "screenrecordercustom.h"
#include <QDebug>

ScreenRecorderCustom::ScreenRecorderCustom(QObject *parent)
    : QObject{parent}
{
}

const AVFrame* ScreenRecorderCustom::frameRGB()
{
    return m_frameRGB;
}

void ScreenRecorderCustom::onFrameUpdate()
{
    qDebug() << "Frame update signal emitted!";
    m_frameRGB = getFrameRGB();
    emit ScreenRecorderCustom::frameUpdated();
}

int ScreenRecorderCustom::initCapture(int windowWidth, int windowHeight, int offsetX, int offsetY, int outputFrameWidth, int outputFrameHeight)
{
    qDebug() << "initCapture!";
    return ScreenRecorder::initCapture(windowWidth, windowHeight, offsetX, offsetY, outputFrameWidth, outputFrameHeight);
}

int ScreenRecorderCustom::startCapture()
{
    qDebug() << "startCapture!";
    return ScreenRecorder::startCapture();
}

int ScreenRecorderCustom::stopCapture()
{
    qDebug() << "stopCapture!";
    return ScreenRecorder::stopCapture();
}
