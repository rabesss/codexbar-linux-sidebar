import QtQuick

// A horizontal usage meter bar.
// usedPercent: 0-100, color: fill color, height: bar thickness
Rectangle {
  id: root

  property real usedPercent: 0
  property color barColor: "#4a9eff"
  property real barHeight: 8
  property bool animate: true

  implicitHeight: root.barHeight
  radius: height / 2
  color: "#1a1a1e"

  // Clip to rounded rect
  clip: true

  Rectangle {
    anchors {
      left: parent.left
      top: parent.top
      bottom: parent.bottom
    }
    width: parent.width * ratio(root.usedPercent)
    radius: parent.radius
    color: root.barColor

    Behavior on width {
      enabled: root.animate
      NumberAnimation { duration: 600; easing.type: Easing.OutCubic }
    }
  }

  function ratio(value) {
    if (value < 0 || isNaN(value)) return 0;
    return Math.min(1, Math.max(0, value / 100));
  }
}
