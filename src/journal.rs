use std::{
    str::FromStr,
};

use journald::{
    reader::{JournalReader, JournalReaderConfig, JournalSeek},
    JournalEntry,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum Severity {
    Error = 0,
    Warning = 5,
    Info = 99,
}

impl FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "error" => Ok(Severity::Error),
            "warning" => Ok(Severity::Warning),
            "info" => Ok(Severity::Info),
            _ => Err(format!("Unknown severity: {}", s)),
        }
    }
}

impl Severity {
    pub fn cardinality(&self) -> usize {
        match self {
            Severity::Error => 0,
            Severity::Warning => 1,
            Severity::Info => 2,
        }
    }

    fn from_priority(it: &str) -> Severity {
        let it = it.parse::<usize>();

        it.map(|it| {
            if it <= 3 {
                Severity::Error
            } else if it <= 5 {
                Severity::Warning
            } else {
                Severity::Info
            }
        })
        .unwrap_or(Severity::Info)
    }
}

#[derive(Debug, Serialize)]
pub struct LogEntry {
    message: String,
    severity: Severity,
    origin: String,
    date: i64,
}

fn create_journal() -> Result<JournalReader, String> {
    let mut journal =
        JournalReader::open(&JournalReaderConfig::default()).expect("journal open failed");
    journal
        .seek(JournalSeek::Tail)
        .map_err(|_| "Cannot initialize journald")?;
    Ok(journal)
}

pub fn query_journal(
    n: &Option<usize>,
    severity: &Option<String>,
) -> Result<Vec<LogEntry>, String> {
    let prio = severity
        .as_ref()
        .and_then(|it| Severity::from_str(it.as_str()).ok())
        .map(|it| it.cardinality())
        .unwrap_or(usize::MAX);

    let entries = JournalReaderIterator::of(create_journal()?)
        .filter(|it| it.severity.cardinality() <= prio)
        .take(n.unwrap_or(100))
        .collect();

    Ok(entries)
}

pub struct JournalReaderIterator {
    reader: JournalReader,
}

impl JournalReaderIterator {
    pub fn of(j: JournalReader) -> JournalReaderIterator {
        JournalReaderIterator { reader: j }
    }
}

impl Iterator for JournalReaderIterator {
    type Item = LogEntry;

    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.previous_entry() {
            Ok(Some(it)) => Some(LogEntry::from_journal_entry(&it)),
            Ok(None) => None,
            Err(err) => {
                println!("Error: {:}", err);
                None
            }
        }
    }
}

impl LogEntry {
    pub fn from_journal_entry(je: &JournalEntry) -> LogEntry {
        let severity = je
            .get_field("PRIORITY")
            .map(|it| Severity::from_priority(it))
            .unwrap_or(Severity::Info);

        let date = je.get_wallclock_time().map_or(0, |it| it.timestamp_us);

        LogEntry {
            message: je.get_message().unwrap_or_default().into(),
            severity,
            date,
            origin: je.get_field("_SYSTEMD_UNIT").unwrap_or_default().into(),
        }
    }
}
