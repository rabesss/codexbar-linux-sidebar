import qs.modules.common
import qs.modules.common.widgets
import qs.modules.common.functions
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import Quickshell.Io

Item {
    id: root

    property var providers: []
    property var errors: []
    property string generatedAt: ""
    property bool hasData: false
    property bool loading: true

    property string statePath: {
        const runtime = Quickshell.env("XDG_RUNTIME_DIR") || ("/run/user/" + (Quickshell.env("UID") || "1000"))
        return runtime + "/codexbar-sidebar/state.json"
    }

    function parseState(raw) {
        if (!raw || raw.length === 0) {
            root.loading = false
            root.hasData = false
            return
        }
        try {
            const state = JSON.parse(raw)
            root.generatedAt = state.generated_at || ""
            root.providers = state.providers || []
            root.errors = state.errors || []
            root.hasData = true
            root.loading = false
        } catch (e) {
            console.warn("CodexBar: failed to parse state.json", e)
            root.loading = false
            root.hasData = false
        }
    }

    function refreshDaemon() {
        refreshProc.running = true
    }

    FileView {
        id: stateFile
        path: root.statePath
        watchChanges: true
        onFileChanged: reload()
        onDataChanged: root.parseState(stateFile.data)
    }

    Timer {
        interval: 5000
        repeat: true
        running: true
        onTriggered: stateFile.reload()
    }

    Process {
        id: refreshProc
        command: ["codexbar-sidebarctl", "refresh"]
        onExited: stateFile.reload()
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 8

        RowLayout {
            Layout.fillWidth: true
            spacing: 8

            StyledText {
                Layout.fillWidth: true
                text: root.loading ? Translation.tr("Loading usage…") : (root.hasData ? Translation.tr("Updated %1").arg(root.generatedAt) : Translation.tr("No usage data yet"))
                font.pixelSize: Appearance.font.size.smaller
                color: Appearance.colors.colSubtext
                elide: Text.ElideRight
            }

            RippleButton {
                implicitWidth: 28
                implicitHeight: 28
                buttonRadius: Appearance.rounding.full
                onPressed: root.refreshDaemon()
                MaterialSymbol {
                    anchors.centerIn: parent
                    text: "refresh"
                    iconSize: Appearance.font.size.larger
                    color: Appearance.colors.colOnLayer1
                }
            }
        }

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
                    delegate: Rectangle {
                        required property var modelData
                        Layout.fillWidth: true
                        implicitHeight: row.implicitHeight + 16
                        radius: Appearance.rounding.normal
                        color: Appearance.colors.colLayer2
                        opacity: modelData.stale ? 0.65 : 1

                        ColumnLayout {
                            id: row
                            anchors.fill: parent
                            anchors.margins: 10
                            spacing: 6

                            RowLayout {
                                Layout.fillWidth: true
                                spacing: 8

                                StyledText {
                                    Layout.fillWidth: true
                                    text: modelData.name || modelData.id || "?"
                                    font.pixelSize: Appearance.font.size.normal
                                    font.bold: true
                                    color: Appearance.colors.colOnLayer1
                                }

                                StyledText {
                                    text: {
                                        if (modelData.platform_state === "unsupported") return Translation.tr("Linux N/A")
                                        const primary = modelData.usage?.primary
                                        return primary?.display_label || "—"
                                    }
                                    font.pixelSize: Appearance.font.size.normal
                                    color: Appearance.colors.colPrimary
                                }
                            }

                            StyledProgressBar {
                                Layout.fillWidth: true
                                visible: modelData.usage?.primary != null
                                from: 0
                                to: 100
                                value: modelData.usage?.primary?.used_percent ?? 0
                            }

                            StyledText {
                                Layout.fillWidth: true
                                visible: modelData.usage?.primary?.reset_label
                                text: Translation.tr("Reset: %1").arg(modelData.usage.primary.reset_label)
                                font.pixelSize: Appearance.font.size.smaller
                                color: Appearance.colors.colSubtext
                                wrapMode: Text.WordWrap
                            }

                            StyledText {
                                Layout.fillWidth: true
                                visible: modelData.unsupported_reason || modelData.auth?.message
                                text: modelData.unsupported_reason || modelData.auth?.message || ""
                                font.pixelSize: Appearance.font.size.smaller
                                color: Appearance.colors.colSubtext
                                wrapMode: Text.WordWrap
                            }
                        }
                    }
                }
            }
        }

        StyledText {
            visible: root.providers.length === 0 && !root.loading
            Layout.fillWidth: true
            text: Translation.tr("Start the daemon with codexbar-sidebarctl start, then refresh.")
            wrapMode: Text.WordWrap
            color: Appearance.colors.colSubtext
        }
    }
}
