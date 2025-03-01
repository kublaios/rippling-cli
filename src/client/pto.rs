use serde::Deserialize;
use serde_json::json;
use time::Date;

use super::session::Session;
use super::Result;

pub fn holiday_calendar(session: &Session) -> Result<Vec<HolidaysOfYear>> {
    session
        .post(&format!("pto/api/get_holiday_calendar/"))
        .send_json(&json!({"allow_time_admin": false, "only_payable": false}))?
        .parse_json()
}

pub fn leave_requests(session: &Session) -> Result<Vec<LeaveRequest>> {
    session
        .get("pto/api/leave_requests/")
        .param("role", session.role().unwrap())
        .send()?
        .parse_json()
}

#[derive(Clone, Debug, Deserialize)]
pub struct HolidaysOfYear {
    pub year: u16,
    pub holidays: Vec<Holiday>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Holiday {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(rename = "startDate")]
    pub start_date: Date,
    #[serde(rename = "endDate")]
    pub end_date: Date,
    #[serde(rename = "shouldCountTowardHoursWorkedForOvertime")]
    pub count_as_overtime: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LeaveRequest {
    #[serde(rename = "isDeleted")]
    pub is_deleted: Option<bool>,
    #[serde(rename = "startDate")]
    pub start_date: Date,
    #[serde(rename = "endDate")]
    pub end_date: Date,
    pub status: String,
    #[serde(rename = "leaveTypeName")]
    pub leave_type_name: String,
}

#[cfg(test)]
mod tests {
    use time::macros::date;
    use utilities::mocking;

    use super::*;

    fn session() -> Session {
        let mut session = Session::new("access-token".into());
        session.set_company_and_role("some-company-id".into(), "some-role-id".into());
        session
    }

    #[test]
    fn it_can_fetch_leave_requests() {
        let _m = mocking::with_fixture("GET", "/pto/api/leave_requests/?role=some-role-id", "leave_requests").create();
        let data = leave_requests(&session()).unwrap();
        assert_eq!(data.len(), 2);
        let days: Vec<Date> = data.into_iter().map(|h| h.start_date).collect();
        assert_eq!(days, vec![date![2022 - 06 - 09], date![2022 - 05 - 23]]);
    }

    #[test]
    fn it_can_fetch_holiday_calendar() {
        let _m = mocking::with_fixture("POST", "/pto/api/get_holiday_calendar/", "holiday_calendar").create();
        let data = holiday_calendar(&session()).unwrap();
        assert_eq!(data.len(), 9);
        let y2023 = data.into_iter().find(|y| y.year == 2023).unwrap();
        assert_eq!(y2023.holidays.len(), 13);
        let days: Vec<Date> = y2023.holidays.into_iter().take(3).map(|h| h.start_date).collect();
        assert_eq!(
            days,
            vec![date![2023 - 01 - 01], date![2023 - 01 - 06], date![2023 - 04 - 07]]
        );
    }
}
