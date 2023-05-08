#ifndef SCREENRECORDERCUSTOM_H
#define SCREENRECORDERCUSTOM_H

#include <QObject>
#include <QList>
#include <QScreen>
#include <QGuiApplication>
#include <QString>
#include "ScreenCap.h"

class ScreenRecorderCustom : public QObject, public ScreenRecorder
{
    Q_OBJECT
    Q_PROPERTY(const AVFrame* frameRGB READ frameRGB CONSTANT)
    Q_PROPERTY(QList<QString> displayList READ displayList CONSTANT)
private:
    const AVFrame* m_frameRGB = nullptr;
    QList<QScreen*> m_displayList;

private:
    void onFrameUpdate() override;

public:
    explicit ScreenRecorderCustom(QObject *parent = nullptr);
    const AVFrame* frameRGB();
    QList<QString> displayList();
    Q_INVOKABLE QRect getDisplayParams(int displayID);
    Q_INVOKABLE bool isInitialized();

signals:
    void frameUpdated();

public slots:
    int initCapture(int windowWidth, int windowHeight, int offsetX, int offsetY, int outputFrameWidth, int outputFrameHeight);
    int startCapture();
    int stopCapture();
};

#endif // SCREENRECORDERCUSTOM_H
