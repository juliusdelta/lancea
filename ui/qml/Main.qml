import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Qt.labs.platform as Platform
import Lancea
import Lancea.System

Window {
    id: win
    width: 640
    height: 420
    visible: true
    title: "Lancea — M0"

    property int currentEpoch: 0
    property int selectedIndex: 0
    ListModel {
        id: resultsModel
    }

    function applyBatch(batchJson) {
        const env = JSON.parse(batchJson);
        const b = env.data;
        if (b.kind === "reset") {
            resultsModel.clear();
            for (let i = 0; i < b.items.length; i++)
                resultsModel.append(b.items[i]);
        } else if (b.kind === "end")
        // stop spinner, etc.
        {}
    }

    Connections {
        target: engineProxy
        // (epoch, providerId, token, batchJson)
        function onResultsUpdated(epoch, providerId, token, batchJson) {
            applyBatch(batchJson);
        }
        function onPreviewUpdated(epoch, providerId, resultKey, previewJson) {
            previewPane.previewJson = previewJson;
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 8

        TextField {
            id: input
            placeholderText: "Type /emoji …"
            focus: true
            KeyNavigation.tab: results
            onTextChanged: debounce.restart()
            onAccepted: {
                if (resultsModel.count > 0) {
                    const item = resultsModel.get(selectedIndex);
                    Clipboard.setText(item.extras?.glyph ?? "");
                    toast.text = "Copied " + item.extras?.glyph;
                    toast.visible = true;
                    toastTimer.restart();
                    // also call execute if desired:
                    engineProxy.execute("copy_glyph", item.key);
                }
            }
        }

        // Debounce 120ms to avoid chatty calls
        // TODO: This is where we'd do some intelligent state management
        // concerning the current command scope of the search.
        Timer {
            id: debounce
            interval: 120
            repeat: false
            running: false
            onTriggered: {
                const t = input.text;
                const resolvedJson = engineProxy.resolveCommand(t);
                const resolved = JSON.parse(resolvedJson).data;
                const providerId = resolved.provider_id ?? "";

                win.currentEpoch += 1;
                engineProxy.search(t, providerId, win.currentEpoch);
            }
        }

        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 12

            ListView {
                id: results
                Layout.fillWidth: true
                Layout.fillHeight: true
                model: resultsModel
                currentIndex: selectedIndex
                Keys.onUpPressed: {
                    if (selectedIndex > 0)
                        selectedIndex -= 1;
                }
                Keys.onDownPressed: {
                    if (selectedIndex < resultsModel.count - 1)
                        selectedIndex += 1;
                }
                onCurrentIndexChanged: {
                    if (currentItem && resultsModel.count > 0) {
                        const item = resultsModel.get(currentIndex);
                        engineProxy.requestPreview(item.key, win.currentEpoch);
                    }
                }
                delegate: Rectangle {
                    width: ListView.view.width
                    height: 40
                    color: (index === results.currentIndex) ? "#2a2f3a" : "transparent"
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.left: parent.left
                        anchors.leftMargin: 8
                        text: model.title + (model.extras?.glyph ? "  " + model.extras.glyph : "")
                    }
                    MouseArea {
                        anchors.fill: parent
                        onClicked: {
                            results.currentIndex = index;
                        }
                        onDoubleClicked: input.accepted()
                    }
                }
            }

            // Preview
            PreviewPane {
                id: previewPane
                Layout.preferredWidth: 220
                Layout.fillHeight: true
            }
        }

        // Toast
        Label {
            id: toast
            visible: false
            text: ""
            horizontalAlignment: Text.AlignHCenter
            Layout.fillWidth: true
            opacity: visible ? 1 : 0
        }
        Timer {
            id: toastTimer
            interval: 1200
            repeat: false
            running: false
            onTriggered: toast.visible = false
        }
    }
}
