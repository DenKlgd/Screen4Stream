#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QDebug>
#include <QQmlContext>
#include "screenrecordercustom.h"
#include "framerenderer.h"

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);
    QQmlApplicationEngine engine;

    ScreenRecorderCustom screenRecorder;

    QQmlContext* rootContext = engine.rootContext();
    rootContext->setContextProperty("screenRecorder", &screenRecorder);

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
