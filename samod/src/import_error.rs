use samod_core::DocumentId;

use crate::Stopped;

pub enum ImportError {
    Stopped,
    AlreadyExists { document_id: DocumentId },
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stopped => write!(f, "the Samod instance has been stopped"),
            Self::AlreadyExists { document_id } => {
                write!(f, "document already exists: {document_id}")
            }
        }
    }
}

impl std::fmt::Debug for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl std::error::Error for ImportError {}

impl From<Stopped> for ImportError {
    fn from(_: Stopped) -> Self {
        Self::Stopped
    }
}
