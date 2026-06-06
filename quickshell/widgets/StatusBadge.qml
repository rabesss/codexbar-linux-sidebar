import QtQuick

// Small status indicator pill.
// level: "ok", "warn", "error", "critical", "unsupported", "maintenance", "unknown"
Rectangle {
  id: root

  property string level: "unknown"
  property string description: ""

  implicitHeight: 20
  implicitWidth: badgeLabel.implicitWidth + 16
  radius: height / 2

  readonly property var levelColors: ({
    "ok": { bg: "#0a2a0a", fg: "#66cc66", border: "#1a4a1a" },
    "warn": { bg: "#2a1a0a", fg: "#ccaa44", border: "#4a3a1a" },
    "error": { bg: "#2a0a0a", fg: "#cc6666", border: "#4a1a1a" },
    "critical": { bg: "#2a0000", fg: "#ff4444", border: "#4a0000" },
    "unsupported": { bg: "#1a1a2a", fg: "#8888cc", border: "#2a2a4a" },
    "maintenance": { bg: "#1a1a1a", fg: "#aaaaaa", border: "#3a3a3a" },
    "unknown": { bg: "#1a1a1a", fg: "#888888", border: "#3a3a3a" }
  })

  readonly property var colors: root.levelColors[root.level] || root.levelColors["unknown"]

  color: colors.bg
  border.color: colors.border
  border.width: 1

  Text {
    id: badgeLabel
    anchors.centerIn: parent
    text: root.level === "unsupported" ? "LINUX" : root.level.toUpperCase()
    color: colors.fg
    font.pixelSize: 10
    font.bold: true
  }

  MouseArea {
    anchors.fill: parent
    hoverEnabled: true
    cursorShape: Qt.WhatsThisCursor

    onEntered: {
      if (root.description) {
        tooltip.text = root.description;
        tooltip.visible = true;
      }
    }
    onExited: {
      tooltip.visible = false;
    }
  }

  // Simple tooltip
  Rectangle {
    id: tooltip
    visible: false
    implicitHeight: tooltipText.implicitHeight + 8
    implicitWidth: tooltipText.implicitWidth + 12
    radius: 4
    color: "#2a2a2e"
    border.color: "#3a3a3e"
    border.width: 1
    z: 100
    x: parent.width + 4
    y: (parent.height - height) / 2

    Text {
      id: tooltipText
      anchors.centerIn: parent
      text: root.description
      color: "#cccccc"
      font.pixelSize: 10
    }
  }
}
