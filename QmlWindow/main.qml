import QtQuick
import QtQuick.Controls 6.3
import QtQuick.Controls.Material 2.3
import QtMultimedia 6.2
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

    Rectangle {
        id: toolbarRect
        width: parent.width * 0.2
        height: parent.height
        color: "#262626"
        Button {
            id: captureSwitch
            width: parent.width * 0.7
            height: 42
            text: qsTr("Начать запись")
            font.pixelSize: toolbarRect.width / 15
            anchors.bottom: parent.bottom
            anchors.horizontalCenter: parent.horizontalCenter
            
            Material.background: Material.Orange

            onClicked: {
                if (!screenRecorder.isInitialized())
                {
                    if (displaySelector.currentIndex < 0)
                        return;
                        
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
        
        Column {
            anchors.top: parent.top
            anchors.horizontalCenter: parent.horizontalCenter
            //leftPadding: parent.width * 0.1

            Row {
                ComboBox {
                    id: displaySelector
                    textRole: "displayName"
                    displayText: "Choose display"
                    width: toolbarRect.width
                    font.pixelSize: toolbarRect.width / 14
                    Material.background: Material.Orange
                    Material.accent: Material.Red

                    model: ListModel {
                        id: displayList

                        Component.onCompleted: {
                            for (var i = 0; i < screenRecorder.displayList.length; i++) {
                                displayList.append( {displayName: '#' + i + " " + screenRecorder.displayList[i]} );
                            }
                        }
                    }

                    onCurrentIndexChanged: {
                        var index = displaySelector.currentIndex;
                        displayText = displayList[index];
                        leftCaptureRectCoord.value = 0;
                        topCaptureRectCoord.value = 0;
                        rightCaptureRectCoord.value = screenRecorder.getDisplayParams(index).width;
                        bottomCaptureRectCoord.value = screenRecorder.getDisplayParams(index).height;
                    }
                }
            }

            Row {
                Label {
                    text: qsTr("Offset X:")
                    color: "orange"
                    font.pixelSize: toolbarRect.width / 14
                    height: 60
                    horizontalAlignment: Text.AlignRight
                    verticalAlignment: Text.AlignVCenter
                    font.bold: true
                }

                SpinBox {
                    id: leftCaptureRectCoord
                    Material.foreground: Material.Orange
                    font.pixelSize: toolbarRect.width / 14
                    width: toolbarRect.width / 1.42
                    height: 60
                    to: 65535
                    editable: true
                }
            }

            Row {
                Label {
                    text: qsTr("Offset Y:")
                    color: "orange"
                    font.pixelSize: toolbarRect.width / 14
                    height: 60
                    horizontalAlignment: Text.AlignRight
                    verticalAlignment: Text.AlignVCenter
                    font.bold: true
                }

                SpinBox {
                    id: topCaptureRectCoord
                    Material.foreground: Material.Orange
                    font.pixelSize: toolbarRect.width / 14
                    width: toolbarRect.width / 1.42
                    height: 60
                    to: 65535
                    editable: true
                }
            }

            Row {
                Label {
                    text: qsTr("Width:")
                    color: "orange"
                    font.pixelSize: toolbarRect.width / 14
                    height: 60
                    horizontalAlignment: Text.AlignRight
                    verticalAlignment: Text.AlignVCenter
                    font.bold: true
                }

                SpinBox {
                    id: rightCaptureRectCoord
                    Material.foreground: Material.Orange
                    font.pixelSize: toolbarRect.width / 12.8
                    width: toolbarRect.width / 1.42
                    height: 60
                    to: 65535
                    editable: true
                }
            }

            Row {
                Label {
                    text: qsTr("Height")
                    color: "orange"
                    font.pixelSize: toolbarRect.width / 12.8
                    height: 60
                    horizontalAlignment: Text.AlignRight
                    verticalAlignment: Text.AlignVCenter
                    font.bold: true
                }

                SpinBox {
                    id: bottomCaptureRectCoord
                    Material.foreground: Material.Orange
                    font.pixelSize: toolbarRect.width / 12.8
                    width: toolbarRect.width / 1.42
                    height: 60
                    to: 65535
                    editable: true
                }
            }
        }
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
