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

QList<QString> ScreenRecorderCustom::displayList()
{
    m_displayList = QGuiApplication::screens();
    QList<QString> jsonDisplayList;

    for (auto& i : m_displayList)
    {
        jsonDisplayList.append(i->name());
    }

    return jsonDisplayList;
}

Q_INVOKABLE QRect ScreenRecorderCustom::getDisplayParams(int displayID)
{
    if (displayID < m_displayList.size())
        return m_displayList.at(displayID)->geometry();
    return QRect(0, 0, 0, 0);
}

Q_INVOKABLE bool ScreenRecorderCustom::isInitialized()
{
    return ScreenRecorder::isInit();
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
