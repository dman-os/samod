use samod_core::DocumentId;

use crate::Stopped;

pub enum LocalExportError {
    Stopped,
    NotFound { document_id: DocumentId },
}

impl std::fmt::Display for LocalExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stopped => write!(f, "the Samod instance has been stopped"),
            Self::NotFound { document_id } => {
                write!(f, "document not found locally: {document_id}")
            }
        }
    }
}

impl std::fmt::Debug for LocalExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl std::error::Error for LocalExportError {}

impl From<Stopped> for LocalExportError {
    fn from(_: Stopped) -> Self {
        Self::Stopped
    }
}
