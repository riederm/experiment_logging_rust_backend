use std::{
    ops::Sub,
    str::FromStr,
    time::{Duration, SystemTime},
};

use journald::{
    reader::{JournalReader, JournalReaderConfig, JournalSeek},
    JournalEntry,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
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
    #[serde(skip)]
    cursor: String,
}

fn create_journal() -> Result<JournalReader, String> {
    let mut journal =
        JournalReader::open(&JournalReaderConfig::default()).expect("journal open failed");
    journal
        .seek(JournalSeek::Tail)
        .map_err(|_| "Cannot initialize journald")?;
    Ok(journal)
}

fn create_journal_at(cursor: &str) -> Result<JournalReader, String> {
    let mut journal =
        JournalReader::open(&JournalReaderConfig::default()).expect("journal open failed");
    journal
        .seek(JournalSeek::Cursor(cursor.into()))
        .map_err(|_| "Cannot initialize journald")?;
    Ok(journal)
}

#[derive(Serialize)]
pub struct QueryResult {
    entries: Vec<LogEntry>,
    last_cursor: String,
}

pub fn query_journal(
    n: &Option<usize>,
    severity: &Option<String>,
    last_secs: &Option<usize>,
    cursor: &Option<String>,
) -> Result<QueryResult, String> {
    let prio = severity
        .as_ref()
        .and_then(|it| Severity::from_str(it.as_str()).ok())
        .map(|it| it.cardinality())
        .unwrap_or(usize::MAX);

    let smallest_ts = last_secs
        .and_then(|last_secs| {
            SystemTime::now()
                .sub(Duration::from_secs(last_secs as u64))
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|it| it.as_micros())
                .ok()
        })
        .unwrap_or(0u128);

    let mut journal = cursor
        .as_ref()
        .map(|c| create_journal_at(c.as_str()))
        .unwrap_or_else(|| create_journal())?;

    let entries: Vec<LogEntry> = JournalReaderIterator::of(&mut journal)
        .filter(|it| it.severity.cardinality() <= prio)
        .take_while(|it| it.date as u128 >= smallest_ts)
        .take(n.unwrap_or(100))
        .collect();

    let last_cursor = entries.last().map(|e| e.cursor.clone()).unwrap_or_default();

    Ok(QueryResult {
        last_cursor,
        entries,
    })
    //Ok(entries)
}

pub struct JournalReaderIterator<'a> {
    reader: &'a mut JournalReader,
}

impl<'a> JournalReaderIterator<'a> {
    pub fn of(j: &'a mut JournalReader) -> JournalReaderIterator<'a> {
        JournalReaderIterator { reader: j }
    }
}

impl<'a> Iterator for JournalReaderIterator<'a> {
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
            cursor: je.get_field("__CURSOR").unwrap_or_default().into(),
        }
    }
}
