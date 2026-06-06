import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import Quickshell.Wayland
import "widgets"

// CodexBar Linux Sidebar — standalone Quickshell left panel
// Shows provider usage, reset timers, credits, cost, and status from CodexBar.

Scope {
  id: root

  readonly property string scriptPath: Quickshell.env("HOME") + "/.local/bin/codexbar-sidebar"
  readonly property string visibilityStatePath: Quickshell.env("HOME") + "/.local/state/codexbar-sidebar/state"

  property bool panelVisible: true

  // --- State properties (populated from state.json) ---
  property var providers: []
  property var codexbarMeta: ({})
  property var errors: []
  property string generatedAt: ""
  property bool hasData: false
  property bool loading: true

  property string statePath: {
    var runtimeDir = Quickshell.env("XDG_RUNTIME_DIR") || ("/run/user/" + (Quickshell.env("UID") || "1000"));
    return runtimeDir + "/codexbar-sidebar/state.json";
  }

  property double lastFileEventMs: 0

  function parseVisibilityState() {
    if (!visibilityFile.loaded) return;
    const lines = visibilityFile.text().split("\n");
    for (const line of lines) {
      if (line.startsWith("visible=")) {
        root.panelVisible = line.slice("visible=".length) === "1";
      }
    }
  }

  function setPanelVisible(value) {
    root.panelVisible = value;
    Quickshell.execDetached([root.scriptPath, "visible", value ? "on" : "off"]);
  }

  FileView {
    id: visibilityFile
    path: root.visibilityStatePath
    onLoadedChanged: root.parseVisibilityState()
  }

  FileView {
    id: stateFile
    path: root.statePath
    watchChanges: true

    onFileChanged: {
      root.lastFileEventMs = Date.now();
      reload();
    }

    onDataChanged: {
      parseState(stateFile.data);
    }
  }

  Timer {
    interval: 5000
    repeat: true
    running: true
    onTriggered: {
      visibilityFile.reload();
      root.parseVisibilityState();
      if (Date.now() - root.lastFileEventMs > 5000) {
        stateFile.reload();
      }
    }
  }

  function parseState(raw) {
    if (!raw || raw.length === 0) {
      root.loading = false;
      root.hasData = false;
      return;
    }

    try {
      var state = JSON.parse(raw);
      root.generatedAt = state.generated_at || "";
      root.codexbarMeta = state.codexbar || {};
      root.providers = state.providers || [];
      root.errors = state.errors || [];
      root.hasData = true;
      root.loading = false;
    } catch (e) {
      console.warn("Failed to parse state.json:", e);
      root.loading = false;
      root.hasData = false;
    }
  }

  // Bootstrap: discover XDG_RUNTIME_DIR if not set in environment
  Process {
    id: envProbe
    command: ["sh", "-c", 'echo "$XDG_RUNTIME_DIR"']
    running: Quickshell.env("XDG_RUNTIME_DIR") === undefined || Quickshell.env("XDG_RUNTIME_DIR") === ""
    stdout: StdioCollector {
      onStreamFinished: {
        var dir = text.trim();
        if (dir !== "") {
          root.statePath = dir + "/codexbar-sidebar/state.json";
          stateFile.path = root.statePath;
          stateFile.reload();
        }
      }
    }
  }

  // --- UI ---
  PanelWindow {
    id: sidebarWindow
    visible: root.panelVisible
    anchors {
      left: true
      top: true
      bottom: true
    }
    width: 430
    exclusiveZone: root.panelVisible ? 430 : 0
    aboveWindows: true

    WlrLayershell.namespace: "codexbar-sidebar"
    WlrLayershell.layer: WlrLayer.Top

    Rectangle {
      anchors.fill: parent
      color: "#0a0a0c"

      ColumnLayout {
        id: mainLayout
        anchors {
          fill: parent
          margins: 12
        }
        spacing: 8

        // Header
        Rectangle {
          Layout.fillWidth: true
          implicitHeight: 40
          color: "transparent"

          RowLayout {
            anchors.fill: parent
            spacing: 8

            Text {
              text: "CodexBar Usage"
              font.pixelSize: 16
              font.bold: true
              color: "#e0e0e0"
              Layout.fillWidth: true
              verticalAlignment: Text.AlignVCenter
            }

            Text {
              text: root.loading ? "Loading..." : (root.hasData ? "Updated " + timeAgo(root.generatedAt) : "No data")
              font.pixelSize: 11
              color: "#888888"
              verticalAlignment: Text.AlignVCenter
              Layout.alignment: Qt.AlignRight
            }

            Rectangle {
              width: 28
              height: 28
              radius: 6
              color: "#1a1a1e"
              border.color: "#2a2a2e"
              border.width: 1

              Text {
                anchors.centerIn: parent
                text: "\u21BB"
                color: "#888888"
                font.pixelSize: 14
              }

              MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: stateFile.reload()
              }
            }
          }
        }

        // CodexBar not available banner
        Rectangle {
          visible: root.hasData && !root.codexbarMeta.available
          Layout.fillWidth: true
          implicitHeight: 40
          radius: 8
          color: "#1a0a0a"
          border.color: "#3a1a1a"
          border.width: 1

          Text {
            anchors.centerIn: parent
            text: "CodexBar CLI not found. Install with: brew install codexbar"
            color: "#cc6666"
            font.pixelSize: 11
          }
        }

        // Provider list
        Flickable {
          Layout.fillWidth: true
          Layout.fillHeight: true
          clip: true
          contentWidth: width
          contentHeight: providerColumn.implicitHeight

          ColumnLayout {
            id: providerColumn
            width: parent.width
            spacing: 8

            Repeater {
              model: root.providers

              ProviderCard {
                Layout.fillWidth: true
                providerData: modelData
              }
            }
          }
        }

        // Footer: errors
        Rectangle {
          visible: root.errors.length > 0
          Layout.fillWidth: true
          implicitHeight: errorsList.implicitHeight + 16
          radius: 8
          color: "#1a1a0a"
          border.color: "#3a3a1a"
          border.width: 1

          ColumnLayout {
            id: errorsList
            anchors {
              fill: parent
              margins: 8
            }
            spacing: 4

            Text {
              text: "Issues (" + root.errors.length + ")"
              font.pixelSize: 11
              font.bold: true
              color: "#ccaa44"
            }

            Repeater {
              model: root.errors
              Text {
                text: "[" + modelData.scope + "] " + (modelData.provider ? modelData.provider + ": " : "") + modelData.message
                font.pixelSize: 10
                color: "#aa8844"
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
              }
            }
          }
        }
      }
    }
  }

  IpcHandler {
    target: "codexbarSidebar"

    function toggle(): void {
      root.setPanelVisible(!root.panelVisible)
    }

    function show(): void {
      root.setPanelVisible(true)
    }

    function hide(): void {
      root.setPanelVisible(false)
    }

    function refresh(): void {
      stateFile.reload()
      Quickshell.execDetached([root.scriptPath, "refresh"])
    }
  }

  // --- Utility functions ---
  function timeAgo(isoString) {
    if (!isoString) return "never";
    var d = new Date(isoString);
    if (isNaN(d.getTime())) return isoString;
    var now = new Date();
    var diffMs = now - d;
    var diffSec = Math.floor(diffMs / 1000);
    if (diffSec < 10) return "just now";
    if (diffSec < 60) return diffSec + "s ago";
    var diffMin = Math.floor(diffSec / 60);
    if (diffMin === 1) return "1m ago";
    if (diffMin < 60) return diffMin + "m ago";
    var diffHr = Math.floor(diffMin / 60);
    if (diffHr === 1) return "1h ago";
    return diffHr + "h ago";
  }

  function clampPercent(value) {
    if (value < 0 || isNaN(value)) return 0;
    return Math.min(100, Math.max(0, value));
  }

  function ratio(value) {
    return clampPercent(value) / 100;
  }
}
