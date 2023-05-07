import QtQuick
import QtMultimedia 6.2
import QtQuick.Controls 6.3
import FFmpegScreenCap.FrameRenderer 1.0

Window {
    id: mainWindow
    width: 1280
    height: 720
    visible: true
    title: qsTr("Screen4Stream")
    minimumWidth: 940
    minimumHeight: 500

    Connections
    {
        target: screenRecorder
        function onFrameUpdated()
        {
            frameRenderer.frameUpdated(screenRecorder.frameRGB);
        }
    }

    Button {
        id: beginCap
        x: 14
        y: 18
        width: 137
        height: 42
        text: qsTr("Начать запись")
        onClicked: {
            //screenRecorder.initCapture(1920, 1080, 0, 0, frameRenderer.width, frameRenderer.height);
            screenRecorder.initCapture(1920, 1080, 0, 0, 1920, 1080);
            screenRecorder.startCapture()
        }
    }

    Button {
        id: stopCap
        x: 14
        y: 79
        width: 137
        height: 42
        text: qsTr("Остановить запись")
        onClicked: screenRecorder.stopCapture()
    }

    Rectangle {
        id: frameRect
        y: 0
        width: parent.width * 0.8
        height: parent.height
        color: "#000000"
        border.color: "#febdad"
        anchors.right: parent.right
        anchors.rightMargin: 0
        layer.textureMirroring: ShaderEffectSource.NoMirroring
        //minimumWidth: 854
        //minimumHeight: 480

        FrameRenderer
        {
            id: frameRenderer
            x: 0
            y: 0
            width: parent.width
            height: parent.height
        }
    }
}
