import QtQuick
import QtQuick.Controls

Item {
  id: root
  property string previewJson: ""
  Rectangle {
    anchors.fill: parent; color: "transparent"
    Column {
      anchors.centerIn: parent
      spacing: 8
      Text {
        id: glyph
        font.pixelSize: 64
        text: {
          if (!root.previewJson) return "";
          const env = JSON.parse(root.previewJson);
          return env.data?.glyph ?? "";
        }
      }
      Text {
        font.pixelSize: 16
        text: {
          if (!root.previewJson) return "";
          const env = JSON.parse(root.previewJson);
          return env.data?.title ?? "";
        }
      }
    }
  }
}
