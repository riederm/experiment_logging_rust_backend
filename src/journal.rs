use journald::{
    reader::{JournalReader, JournalReaderConfig, JournalSeek},
    JournalEntry,
};
use serde::{Serialize};

#[derive(Debug, Serialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Serialize)]
pub struct LogEntry {
    message: String,
    severity: Severity,
    origin: String,
    date: i64,
}



pub fn read_last_n_entries(n: usize) -> Result<Vec<LogEntry>, String> {
    let mut journal =
        JournalReader::open(&JournalReaderConfig::default()).expect("journal open failed");
    journal.seek(JournalSeek::Tail).unwrap();

    let mut current_entry = journal
        .previous_entry()
        .map_err(|_| "cannot retrieve journal entry".to_string())?;
    let mut entries = Vec::new();
    while let Some(ce) = current_entry {
        let e = LogEntry::from_journal_entry(&ce);
        entries.push(e);

        current_entry = if entries.len() >= n {
            None
        } else {
            journal
                .previous_entry()
                .map_err(|_| "cannot retrieve journal entry".to_string())?
        }
    }

    Ok(entries)
}

impl LogEntry {
    pub fn from_journal_entry(je: &JournalEntry) -> LogEntry {
        let severity = je
            .get_field("PRIORITY")
            .and_then(|it| it.parse::<i32>().ok())
            .map(|it| {
                if it <= 3 {
                    Severity::Error
                } else if it <= 5 {
                    Severity::Warning
                } else {
                    Severity::Info
                }
            })
            .unwrap_or(Severity::Info);

        LogEntry {
            message: je.get_message().unwrap_or_default().into(),
            severity,
            date: je.get_wallclock_time().map_or(0, |it| it.timestamp_us),
            origin: je.get_field("_SYSTEMD_UNIT").unwrap_or_default().into(),
        }
    }
}
