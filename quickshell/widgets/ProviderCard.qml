import QtQuick
import QtQuick.Layouts

// A card displaying a single provider's usage, status, credits, and reset info.
Rectangle {
  id: root

  property var providerData: ({})
  property real preferredHeight: cardLayout.implicitHeight + 20

  implicitHeight: root.preferredHeight
  radius: 10
  color: "#141416"
  border.color: "#1e1e22"
  border.width: 1

  // Derived properties from providerData
  readonly property string providerId: root.providerData.id || ""
  readonly property string providerName: root.providerData.name || root.providerId
  readonly property bool isUnsupported: (root.providerData.platform_state || "") === "unsupported"
  readonly property bool isStale: root.providerData.stale || false
  readonly property bool hasData: root.providerData.usage !== null && root.providerData.usage !== undefined

  readonly property var statusInfo: root.providerData.status || ({})
  readonly property var usageInfo: root.providerData.usage || ({})
  readonly property var primaryUsage: usageInfo.primary || null
  readonly property var creditsInfo: root.providerData.credits || null
  readonly property var authInfo: root.providerData.auth || null
  readonly property var costInfo: root.providerData.cost || null

  readonly property string statusLevel: statusInfo.level || "unknown"

  // Opacity for stale data
  opacity: root.isStale ? 0.6 : 1.0

  ColumnLayout {
    id: cardLayout
    anchors {
      fill: parent
      margins: 12
    }
    spacing: 8

    // Row 1: Provider icon + name + status badge
    RowLayout {
      Layout.fillWidth: true
      spacing: 8

      // Provider icon placeholder
      Rectangle {
        width: 28
        height: 28
        radius: 6
        color: providerIconColor(root.providerId)

        Text {
          anchors.centerIn: parent
          text: root.providerName.charAt(0).toUpperCase()
          color: "#ffffff"
          font.pixelSize: 13
          font.bold: true
        }
      }

      Text {
        text: root.providerName
        font.pixelSize: 14
        font.bold: true
        color: root.isStale ? "#888888" : "#e0e0e0"
        Layout.fillWidth: true
        verticalAlignment: Text.AlignVCenter
      }

      StatusBadge {
        level: root.statusLevel
        description: statusInfo.description || ""
        Layout.alignment: Qt.AlignRight
      }
    }

    // Row 2: Usage bar (if available)
    Rectangle {
      visible: root.primaryUsage !== null && !root.isUnsupported
      Layout.fillWidth: true
      implicitHeight: 28
      color: "transparent"

      RowLayout {
        anchors.fill: parent
        spacing: 8

        ColumnLayout {
          Layout.fillWidth: true
          spacing: 4

          UsageBar {
            Layout.fillWidth: true
            usedPercent: root.primaryUsage ? root.primaryUsage.used_percent : 0
            barColor: usageBarColor(root.primaryUsage ? root.primaryUsage.used_percent : 0)
            barHeight: 10
          }

          // Reset info
          Text {
            visible: root.primaryUsage && root.primaryUsage.reset_label
            text: "Reset: " + (root.primaryUsage ? (root.primaryUsage.reset_label || "") : "")
            font.pixelSize: 10
            color: "#777777"
          }
        }

        // Percentage label
        Text {
          visible: root.primaryUsage !== null
          text: root.primaryUsage ? root.primaryUsage.display_label : ""
          font.pixelSize: 18
          font.bold: true
          color: usageLabelColor(root.primaryUsage ? root.primaryUsage.used_percent : 0)
          verticalAlignment: Text.AlignVCenter
          Layout.alignment: Qt.AlignRight
        }
      }
    }

    // Row 3: Secondary usage (weekly/secondary window)
    Rectangle {
      visible: usageInfo.secondary && !root.isUnsupported
      Layout.fillWidth: true
      implicitHeight: 22
      color: "transparent"

      RowLayout {
        anchors.fill: parent
        spacing: 8

        UsageBar {
          Layout.fillWidth: true
          usedPercent: usageInfo.secondary ? usageInfo.secondary.used_percent : 0
          barColor: "#6b7280"
          barHeight: 4
        }

        Text {
          text: usageInfo.secondary ? usageInfo.secondary.display_label : ""
          font.pixelSize: 10
          color: "#888888"
        }
      }
    }

    // Row 4: Credits / Spend (if available)
    Rectangle {
      visible: (root.creditsInfo !== null || root.costInfo !== null) && !root.isUnsupported
      Layout.fillWidth: true
      implicitHeight: 18
      color: "transparent"

      RowLayout {
        anchors.fill: parent
        spacing: 12

        Text {
          visible: root.creditsInfo !== null && root.creditsInfo.remaining !== null
          text: {
            if (!root.creditsInfo || root.creditsInfo.remaining === null) return "";
            var currency = root.creditsInfo.currency || "";
            var prefix = currency ? "$" : "";
            return prefix + root.creditsInfo.remaining.toFixed(2) + (currency ? " " + currency : " credits");
          }
          font.pixelSize: 11
          color: "#aaaaaa"
        }

        Text {
          visible: root.costInfo !== null && root.costInfo.last_30_days_cost_usd !== null
          text: {
            if (!root.costInfo || root.costInfo.last_30_days_cost_usd === null) return "";
            return "Spent (30d): $" + root.costInfo.last_30_days_cost_usd.toFixed(2);
          }
          font.pixelSize: 11
          color: "#888888"
          Layout.alignment: Qt.AlignRight
          Layout.fillWidth: true
          horizontalAlignment: Text.AlignRight
        }
      }
    }

    // Row 5: Auth message / error
    Rectangle {
      visible: root.authInfo && root.authInfo.message && root.authInfo.message !== ""
      Layout.fillWidth: true
      implicitHeight: 18
      color: "transparent"

      Text {
        anchors.fill: parent
        text: (root.authInfo ? root.authInfo.message : "") || ""
        font.pixelSize: 10
        color: root.isUnsupported ? "#8888cc" : "#ccaa44"
        elide: Text.ElideRight
      }
    }

    // Row 6: Unsupported details
    Rectangle {
      visible: root.isUnsupported && root.providerData.unsupported_reason
      Layout.fillWidth: true
      implicitHeight: 18
      color: "transparent"

      Text {
        anchors.fill: parent
        text: root.providerData.unsupported_reason || ""
        font.pixelSize: 10
        color: "#666688"
        elide: Text.ElideRight
      }
    }
  }

  // --- Color helpers ---
  function providerIconColor(id) {
    var colors = {
      "codex": "#4a9eff",
      "claude": "#cc9966",
      "cursor": "#66cc66",
      "openrouter": "#cc6644",
      "gemini": "#6699ff",
      "copilot": "#8844cc",
      "minimax": "#44cccc",
      "grok": "#cc44cc",
      "openai": "#44cc88",
      "deepseek": "#4488ff",
      "kimi": "#ff6644"
    };
    return colors[id] || "#555555";
  }

  function usageBarColor(percent) {
    if (percent >= 85) return "#cc4444";
    if (percent >= 60) return "#ccaa44";
    return "#4a9eff";
  }

  function usageLabelColor(percent) {
    if (percent >= 85) return "#cc4444";
    if (percent >= 60) return "#ccaa44";
    return "#4a9eff";
  }
}
