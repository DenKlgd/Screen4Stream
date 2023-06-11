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
            videoStreamer.sendVideoFrame(screenRecorder.frameRGB);
        }
    }

    Connections
    {
        target: videoStreamer
        function onFrameReceived()
        {
            frameRenderer.frameUpdated(videoStreamer.frameRGB);
        }
    }

    Rectangle {
        id: toolbarRect
        width: parent.width * 0.2
        height: parent.height
        color: "#262626"
        
        Column {
            id: streamSettings
            anchors.top: parent.top
            anchors.horizontalCenter: parent.horizontalCenter

            Row {
                anchors.horizontalCenter: parent.horizontalCenter
                ComboBox {
                    id: displaySelector
                    textRole: "displayName"
                    displayText: "Choose display"
                    width: toolbarRect.width * 0.8
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

                        var width = screenRecorder.getDisplayParams(index).width;
                        var height = screenRecorder.getDisplayParams(index).height;
                        
                        leftCaptureRectCoord.to = width - 1;
                        topCaptureRectCoord.to = height - 1;
                        rightCaptureRectCoord.to = width;
                        bottomCaptureRectCoord.to = height;

                        leftCaptureRectCoord.value = 0;
                        topCaptureRectCoord.value = 0;
                        rightCaptureRectCoord.value = width;
                        bottomCaptureRectCoord.value = height;

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
                    text: qsTr("Height:")
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

            Row {
                anchors.horizontalCenter: parent.horizontalCenter
                topPadding: 40

                Button {
                    id: captureSwitch
                    height: 55
                    text: qsTr("Начать запись")
                    font.pixelSize: toolbarRect.width / 15
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

                            if (rightCaptureRectCoord.value <= offsetX || bottomCaptureRectCoord.value <= offsetY)
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
            }
        }

        Column {
            id: connectionSettings
            anchors.top: parent.top
            anchors.horizontalCenter: parent.horizontalCenter
            visible: false

            Row {
                Label {
                    text: "IP-address:"
                    color: "orange"
                    font.pixelSize: toolbarRect.width / 13
                    font.bold: true
                    horizontalAlignment: Text.AlignRight
                    verticalAlignment: Text.AlignBottom 
                }

                TextField {
                    color: "orange"
                    font.pixelSize: toolbarRect.width / 13
                    inputMask: "000.000.000.000;_"
                    Material.accent: Material.Orange
                }
            }

            Row {
                anchors.horizontalCenter: parent.horizontalCenter
                Button {
                    id: connectSwitch
                    height: 55
                    text: qsTr("Подключиться")
                    font.pixelSize: toolbarRect.width / 14
                    Material.background: Material.Orange

                    onClicked: {
                        if (!videoStreamer.getSocketState())
                        {
                            if (!videoStreamer.openSocket())
                                return;
                            
                            connectSwitch.text = qsTr("Отключиться");
                        }
                        else 
                        {
                            videoStreamer.closeSocket();
                            connectSwitch.text = qsTr("Подключиться");
                        }
                    }
                }
            }
        }

        Switch {
            id: modeSwitch
            Material.accent: Material.Orange
            font.pixelSize: toolbarRect.width / 20
            font.bold: true
            anchors.bottom: parent.bottom
            anchors.horizontalCenter: parent.horizontalCenter

            contentItem: Rectangle { 
                Text {
                    id: switchTextLeft
                    text: "Демонстрация"
                    font: modeSwitch.font
                    opacity: enabled ? 1.0 : 0.3
                    color: "Orange"
                    verticalAlignment: Text.AlignVCenter
                    x: modeSwitch.indicator.x - switchTextLeft.width - 20
                    //leftPadding: modeSwitch.indicator.x - modeSwitch.indicator.width + switchTextLeft.width + modeSwitch.spacing
                }
                Text {
                    id: switchTextRight
                    text: "Подключение"                           
                    font: modeSwitch.font
                    opacity: enabled ? 1.0 : 0.3
                    color: "White"
                    verticalAlignment: Text.AlignVCenter
                    x: modeSwitch.indicator.width - 10
                    //leftPadding: modeSwitch.indicator.width + modeSwitch.spacing
                }
            }

            MouseArea {
                x: modeSwitch.indicator.x
                y: modeSwitch.indicator.y
                width: modeSwitch.indicator.width
                height: modeSwitch.indicator.height
                onClicked: {
                    modeSwitch.checked = !modeSwitch.checked;
                    if (modeSwitch.checked)
                    {
                        screenRecorder.stopCapture();
                        switchTextRight.color = "orange";
                        switchTextLeft.color = "white";
                        streamSettings.visible = false;
                        connectionSettings.visible = true;
                    }
                    else
                    {
                        switchTextRight.color = "white";
                        switchTextLeft.color = "orange";
                        streamSettings.visible = true;
                        connectionSettings.visible = false;
                    }
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
