#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QUdpSocket>
#include <QDebug>
#include "screenrecordercustom.h"
#include "framerenderer.h"
#include "UdpSocketCustom.h"

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);
    QQmlApplicationEngine engine;

    ScreenRecorderCustom screenRecorder;
    UdpSocketCustom udpSocket;

    QQmlContext* rootContext = engine.rootContext();
    rootContext->setContextProperty("screenRecorder", &screenRecorder);
    rootContext->setContextProperty("videoStreamer", &udpSocket);

    qmlRegisterType<FrameRenderer>("FFmpegScreenCap.FrameRenderer", 1, 0, "FrameRenderer");


    const QUrl url(u"qrc:/QmlWindow/main.qml"_qs);
    QObject::connect(&engine, &QQmlApplicationEngine::objectCreated,
                     &app, [url](QObject *obj, const QUrl &objUrl) {
        if (!obj && url == objUrl)
            QCoreApplication::exit(-1);
    }, Qt::QueuedConnection);
    engine.load(url);

    return app.exec();
}
