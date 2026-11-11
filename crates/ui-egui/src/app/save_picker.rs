//! Asking where to save: the host's Save picker, answered at once (`Services::pick_save`, the
//! desktop's system dialog) or later (`Services::pick_save_async`, a host whose picker is another
//! activity or window that the interface thread can't wait for, such as Android). Either way the
//! same step follows the answer: Save As, an export, or Save a Copy.

use std::sync::mpsc::{Receiver, TryRecvError, channel};

use super::{Json, SheetApp, clean_error, json};

/// What follows the picker's answer.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AfterPick {
    /// Save As (xlsx): the workbook is saved to the chosen file, `.xlsx` added when it has no
    /// extension.
    SaveAs,
    /// File › Export as PDF: `file.exportPdf` with these params plus the chosen path.
    ExportPdf(Json),
    /// The Backstage view's exports: `id` writes `fmt` to the chosen path (its extension made `fmt`).
    Export { id: &'static str, fmt: String },
    /// Save a Copy: `id` writes to the chosen path as it is.
    SaveCopy { id: &'static str },
}

/// A Save picker that hasn't answered yet.
pub(crate) struct PendingPick {
    answer: Receiver<String>,
    then: AfterPick,
}

/// How a request to ask where to save went.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Picking {
    /// The picker answered at once and the step ran: whether it succeeded.
    Done(bool),
    /// The picker is open; the step runs when it answers ([`SheetApp::poll_save_picker`]).
    Waiting,
    /// The user cancelled.
    Cancelled,
    /// The host has no Save picker.
    NoPicker,
}

impl SheetApp {
    /// Whether the host can ask where to save (the desktop's dialog, or a picker that answers later).
    pub(crate) fn has_save_picker(&self) -> bool {
        self.services.pick_save.is_some() || self.services.pick_save_async.is_some()
    }

    /// Asks where to save `suggested`, then runs `then` with the answer: now, when the picker
    /// answers at once, or from a later frame's [`logic`](SheetApp::logic). A newer request
    /// replaces one still waiting.
    pub(crate) fn pick_save_then(&mut self, suggested: &str, then: AfterPick) -> Picking {
        if let Some(start) = &self.services.pick_save_async {
            let (reply, answer) = channel();
            start(suggested, reply);
            self.pending_save = Some(PendingPick { answer, then });
            return Picking::Waiting;
        }
        let Some(pick) = &self.services.pick_save else { return Picking::NoPicker };
        match pick(suggested) {
            Some(path) => Picking::Done(self.after_pick(&path, then)),
            None => Picking::Cancelled,
        }
    }

    /// A late Save picker's answer: runs its step once it has come. A picker closed without an
    /// answer (its sender dropped) is forgotten.
    pub(crate) fn poll_save_picker(&mut self) {
        let Some(pending) = &self.pending_save else { return };
        match pending.answer.try_recv() {
            Ok(path) => {
                if let Some(pending) = self.pending_save.take() {
                    // What the desktop does when its dialog returns: the Backstage view closes.
                    if self.after_pick(&path, pending.then) {
                        self.backstage = None;
                    }
                    self.after_engine();
                }
            }
            Err(TryRecvError::Disconnected) => self.pending_save = None,
            Err(TryRecvError::Empty) => {}
        }
    }

    /// Runs the step that follows the picker's answer `path`. Returns whether it succeeded;
    /// failures are shown.
    fn after_pick(&mut self, path: &str, then: AfterPick) -> bool {
        let with_ext = |ext: &str| std::path::Path::new(path).with_extension(ext).to_string_lossy().into_owned();
        // The Backstage view's exports show their error as a message, the rest as an alert.
        let backstage_export = matches!(then, AfterPick::Export { .. });
        let (id, params) = match then {
            AfterPick::SaveAs => {
                let path = if std::path::Path::new(path).extension().is_none() { format!("{path}.xlsx") } else { path.to_owned() };
                ("file.save", json!({"path": path}))
            }
            // The picker may suggest a workbook extension; the export writes a PDF.
            AfterPick::ExportPdf(mut params) => {
                let path = with_ext("pdf");
                match &mut params {
                    Json::Object(map) => {
                        map.insert("path".into(), Json::String(path));
                    }
                    other => *other = json!({"path": path}),
                }
                ("file.exportPdf", params)
            }
            AfterPick::Export { id, fmt } => (id, json!({"path": with_ext(&fmt)})),
            AfterPick::SaveCopy { id } => (id, json!({"path": path})),
        };
        match self.run(id, params) {
            Ok(_) => true,
            Err(e) if backstage_export => {
                self.message = Some(("GridCraft".into(), clean_error(&e)));
                false
            }
            Err(e) => {
                self.alert(e);
                false
            }
        }
    }
}
