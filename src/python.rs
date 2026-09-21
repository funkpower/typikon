//! Python bindings.
//!
//! These mirror `DayJson` rather than `Day`. `DayJson` is already documented as
//! the wire format front ends read, and it is the shape that has to stay stable
//! for them; binding the internal struct instead would publish field names that
//! are free to move and would leave Python and JSON disagreeing about the same
//! day. So the getters below are the JSON fields, spelled the way Python spells
//! things.
//!
//! Built only under the `python` feature, which maturin turns on. See Cargo.toml
//! for why it cannot be on by default.

use pyo3::exceptions::{PyValueError, PyOSError};
use pyo3::prelude::*;
use pyo3::types::{PyDate, PyDict, PyList};


use crate::calendars::Date;
use crate::day;
use crate::fasting::Obligation;
use crate::jurisdictions::{self, Jurisdiction};

/// A `datetime.date` to the crate's julian-day-number date.
///
/// Read through attributes rather than pyo3's `PyDateAccess`, which the limited
/// API does not expose — the abi3 build that gets us one wheel per platform
/// instead of one per platform and interpreter cannot see datetime's C struct.
/// Constructing a date is fine either way, because pyo3 calls Python's own
/// constructor for that. Going through attributes also means anything shaped
/// like a date works, `datetime.datetime` included.
fn to_date(d: &Bound<'_, PyDate>) -> PyResult<Date> {
    let y: i32 = d.getattr("year")?.extract()?;
    let m: u32 = d.getattr("month")?.extract()?;
    let day: u32 = d.getattr("day")?.extract()?;
    Date::from_ymd(y, m, day)
        .ok_or_else(|| PyValueError::new_err(format!("date out of range: {y}-{m:02}-{day:02}")))
}

/// Back the other way, for anything that hands out a date.
fn from_date<'py>(py: Python<'py>, d: Date) -> PyResult<Bound<'py, PyDate>> {
    let (y, m, day) = d.ymd();
    PyDate::new(py, y, m as u8, day as u8)
}

/// The jurisdiction a call should reckon by, resolved the same way the command
/// line resolves it: the named one or the remembered default, with the
/// remembered usage and discipline applied on top.
fn resolve(
    jurisdiction: Option<&str>,
    tradition: Option<&str>,
    typikon: Option<bool>,
) -> PyResult<&'static Jurisdiction> {
    jurisdictions::resolve_full(jurisdiction, tradition, typikon).map_err(PyValueError::new_err)
}

/// One day, fully reckoned.
///
/// Immutable: it is the answer to a question about a date, not a record to edit.
#[pyclass(frozen, module = "typikon", name = "Day")]
pub struct PyDay {
    inner: day::Day,
}

#[pymethods]
impl PyDay {
    /// The civil date.
    #[getter]
    fn date<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDate>> {
        from_date(py, self.inner.date)
    }

    /// The civil date as an ISO string, matching the JSON.
    #[getter]
    fn iso(&self) -> String {
        self.inner.date.iso()
    }

    #[getter]
    fn weekday(&self) -> &'static str {
        self.inner.weekday.name()
    }

    #[getter]
    fn jurisdiction(&self) -> &'static str {
        self.inner.jurisdiction
    }

    #[getter]
    fn calendar(&self) -> &'static str {
        self.inner.calendar
    }

    /// The ecclesiastical date, as "25 December".
    #[getter]
    fn church_date(&self) -> &str {
        &self.inner.church_date
    }

    #[getter]
    fn season(&self) -> Option<&'static str> {
        self.inner.season.map(|s| s.name())
    }

    #[getter]
    fn cycle(&self) -> Option<&str> {
        self.inner.cycle.as_deref()
    }

    #[getter]
    fn week(&self) -> Option<&str> {
        self.inner.week.as_deref()
    }

    #[getter]
    fn feast(&self) -> Option<&str> {
        self.inner.feast.as_deref()
    }

    #[getter]
    fn rank(&self) -> Option<&'static str> {
        self.inner.rank.map(|r| r.key())
    }

    /// The mark the liturgical books print beside a day of this rank.
    #[getter]
    fn sign(&self) -> Option<&'static str> {
        self.inner.rank.map(|r| r.sign())
    }

    /// Greek or Slavic usage, as this jurisdiction reads the Typikon.
    #[getter]
    fn tradition(&self) -> &'static str {
        self.inner.tradition
    }

    /// The feast if there is one, else the principal commemoration.
    #[getter]
    fn title(&self) -> &str {
        self.inner.title()
    }

    #[getter]
    fn commemorations(&self) -> Vec<String> {
        self.inner.commemorations.clone()
    }

    #[getter]
    fn fast(&self) -> &'static str {
        self.inner.fast.key()
    }

    #[getter]
    fn fast_label(&self) -> &'static str {
        self.inner.fast_label
    }

    #[getter]
    fn emoji(&self) -> &'static str {
        self.inner.emoji
    }

    #[getter]
    fn reason(&self) -> &str {
        &self.inner.reason
    }

    #[getter]
    fn obligation(&self) -> Option<&'static str> {
        self.inner.obligation.map(|o| match o {
            Obligation::Obligatory => "obligatory",
            Obligation::Customary => "customary",
        })
    }

    #[getter]
    fn binds(&self) -> Option<&'static str> {
        self.inner.binds.map(|b| b.key())
    }

    #[getter]
    fn pascha<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDate>> {
        from_date(py, self.inner.pascha)
    }

    #[getter]
    fn days_to_pascha(&self) -> i32 {
        self.inner.days_to_pascha
    }

    #[getter]
    fn julian_offset(&self) -> i32 {
        self.inner.julian_offset
    }

    /// True when the day's fast binds rather than merely being customary.
    fn is_obligatory(&self) -> bool {
        self.inner.is_obligatory()
    }

    /// The same fields as `typikon --json`, as a dict.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let j = self.inner.to_json();
        let d = PyDict::new(py);
        d.set_item("date", &j.date)?;
        d.set_item("weekday", j.weekday)?;
        d.set_item("jurisdiction", j.jurisdiction)?;
        d.set_item("calendar", j.calendar)?;
        d.set_item("church_date", j.church_date)?;
        d.set_item("season", j.season)?;
        d.set_item("cycle", j.cycle)?;
        d.set_item("week", j.week)?;
        d.set_item("feast", j.feast)?;
        d.set_item("rank", j.rank)?;
        d.set_item("sign", j.sign)?;
        d.set_item("tradition", j.tradition)?;
        d.set_item("title", j.title)?;
        d.set_item("commemorations", j.commemorations)?;
        d.set_item("fast", j.fast)?;
        d.set_item("fast_label", j.fast_label)?;
        d.set_item("emoji", j.emoji)?;
        d.set_item("reason", j.reason)?;
        d.set_item("obligation", j.obligation)?;
        d.set_item("binds", j.binds)?;
        d.set_item("pascha", &j.pascha)?;
        d.set_item("days_to_pascha", j.days_to_pascha)?;
        d.set_item("julian_offset", j.julian_offset)?;
        Ok(d)
    }

    fn __repr__(&self) -> String {
        format!(
            "<typikon.Day {} {} fast={}>",
            self.inner.date.iso(),
            self.inner.title(),
            self.inner.fast.key()
        )
    }

    fn __str__(&self) -> String {
        let t = self.inner.title();
        if t.is_empty() {
            format!("{} — {}", self.inner.date.iso(), self.inner.fast_label)
        } else {
            format!("{} — {}", self.inner.date.iso(), t)
        }
    }
}

/// Reckon one civil date.
#[pyfunction]
#[pyo3(signature = (date, jurisdiction = None, tradition = None, typikon = None))]
fn describe(
    date: &Bound<'_, PyDate>,
    jurisdiction: Option<&str>,
    tradition: Option<&str>,
    typikon: Option<bool>,
) -> PyResult<PyDay> {
    let jur = resolve(jurisdiction, tradition, typikon)?;
    Ok(PyDay { inner: day::describe(to_date(date)?, jur) })
}

/// Reckon today, by the system clock in local time.
#[pyfunction]
#[pyo3(signature = (jurisdiction = None, tradition = None, typikon = None))]
fn today(
    py: Python<'_>,
    jurisdiction: Option<&str>,
    tradition: Option<&str>,
    typikon: Option<bool>,
) -> PyResult<PyDay> {
    // Python owns the notion of "local", and it already handles the timezone
    // database. Asking it is both shorter and more correct than reimplementing
    // the lookup here.
    let date = py
        .import("datetime")?
        .getattr("date")?
        .call_method0("today")?;
    let date = date.cast::<PyDate>().map_err(|_| {
        PyOSError::new_err("datetime.date.today() did not return a date")
    })?;
    describe(date, jurisdiction, tradition, typikon)
}

/// Reckon a run of consecutive days starting at `start`.
#[pyfunction]
#[pyo3(signature = (start, days = 7, jurisdiction = None, tradition = None, typikon = None))]
fn week(
    start: &Bound<'_, PyDate>,
    days: i32,
    jurisdiction: Option<&str>,
    tradition: Option<&str>,
    typikon: Option<bool>,
) -> PyResult<Vec<PyDay>> {
    if days < 1 {
        return Err(PyValueError::new_err("days must be at least 1"));
    }
    let jur = resolve(jurisdiction, tradition, typikon)?;
    Ok(day::week(to_date(start)?, days, jur)
        .into_iter()
        .map(|inner| PyDay { inner })
        .collect())
}

/// The date of Pascha in a given year.
#[pyfunction]
#[pyo3(signature = (year, jurisdiction = None))]
fn pascha<'py>(
    py: Python<'py>,
    year: i32,
    jurisdiction: Option<&str>,
) -> PyResult<Bound<'py, PyDate>> {
    // Pascha depends on the paschalion alone; tradition and discipline cannot
    // move it, so they are not arguments here.
    let jur = resolve(jurisdiction, None, None)?;
    let (movables, _) = day::year_of(year, jur);
    from_date(py, movables.pascha)
}

/// Every jurisdiction this build knows, as dicts.
#[pyfunction]
#[pyo3(name = "jurisdictions")]
fn list_jurisdictions<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
    let out = PyList::empty(py);
    for j in jurisdictions::JURISDICTIONS.iter().chain(jurisdictions::CALENDARS) {
        let d = PyDict::new(py);
        d.set_item("key", j.key)?;
        d.set_item("name", j.name)?;
        d.set_item("note", j.note)?;
        out.append(d)?;
    }
    Ok(out)
}

#[pymodule]
fn _typikon(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__doc__", "The Orthodox liturgical day: feasts, seasons and the fast, computed rather than fetched.")?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<PyDay>()?;
    m.add_function(wrap_pyfunction!(describe, m)?)?;
    m.add_function(wrap_pyfunction!(today, m)?)?;
    m.add_function(wrap_pyfunction!(week, m)?)?;
    m.add_function(wrap_pyfunction!(pascha, m)?)?;
    m.add_function(wrap_pyfunction!(list_jurisdictions, m)?)?;
    Ok(())
}
