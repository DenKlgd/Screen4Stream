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
        id: captureSwitch
        x: 14
        y: 18
        width: 137
        height: 42
        text: qsTr("Начать запись")
        onClicked: {
            if (!screenRecorder.isInitialized())
            {
                var screenRect = screenRecorder.getDisplayParams(displaySelector.currentIndex);
                var offsetX = screenRect.left + leftCaptureRectCoord.value;
                var offsetY = screenRect.top + topCaptureRectCoord.value;
                var width = rightCaptureRectCoord.value - leftCaptureRectCoord.value
                var height = bottomCaptureRectCoord.value - topCaptureRectCoord.value

                if (width <= offsetX || height <= offsetY)
                    return;

                screenRecorder.initCapture(width, height, offsetX, offsetY, 1920, 1080);
                screenRecorder.startCapture();

                captureSwitch.text = qsTr("Остановить запись");
            }
            else
            {
                screenRecorder.stopCapture();
                captureSwitch.text = qsTr("Начать запись");
            }
        }
    }

    ComboBox {
        id: displaySelector
        textRole: "displayName"
        displayText: "Choose display"
        x: 100
        y: 300
        model: ListModel {
            id: model

            Component.onCompleted: {
                for (var i = 0; i < screenRecorder.displayList.length; i++) {
                    model.append( {displayName: '#' + i + " " + screenRecorder.displayList[i]} );
                }
            }
        }
        onCurrentIndexChanged: {
            var index = displaySelector.currentIndex;
            displayText = model[index];
            leftCaptureRectCoord.value = 0;
            topCaptureRectCoord.value = 0;
            rightCaptureRectCoord.value = screenRecorder.getDisplayParams(index).width;
            bottomCaptureRectCoord.value = screenRecorder.getDisplayParams(index).height;
        }
    }

    SpinBox {
        id: leftCaptureRectCoord
        x: 40
        y: 400
        width: 60
        height: 40
        to: 65535
        editable: true
    }

    SpinBox {
        id: topCaptureRectCoord
        x: 100
        y: 400
        width: 60
        height: 40
        to: 65535
        editable: true
    }

    SpinBox {
        id: rightCaptureRectCoord
        x: 40
        y: 450
        width: 60
        height: 40
        to: 65535
        editable: true
    }

    SpinBox {
        id: bottomCaptureRectCoord
        x: 100
        y: 450
        width: 60
        height: 40
        to: 65535
        editable: true
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
