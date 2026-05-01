//! Tests for the OFX 1.x SGML / OFX 2.x XML parser (TXN-05, D-19).

use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::str::FromStr;

use super::ofx_parser::{direction_from_amount, parse_ofx, OfxTransaction};

const SGML_SAMPLE: &str = "OFXHEADER:100\r\n\
DATA:OFXSGML\r\n\
VERSION:102\r\n\
SECURITY:NONE\r\n\
ENCODING:USASCII\r\n\
CHARSET:1252\r\n\
COMPRESSION:NONE\r\n\
OLDFILEUID:NONE\r\n\
NEWFILEUID:NONE\r\n\
\r\n\
<OFX>\r\n\
<BANKMSGSRSV1>\r\n\
<STMTTRNRS>\r\n\
<STMTRS>\r\n\
<BANKACCTFROM>\r\n\
<BANKID>123456789\r\n\
<ACCTID>987654321\r\n\
<ACCTTYPE>CHECKING\r\n\
</BANKACCTFROM>\r\n\
<BANKTRANLIST>\r\n\
<DTSTART>20260401\r\n\
<DTEND>20260430\r\n\
<STMTTRN>\r\n\
<TRNTYPE>CREDIT\r\n\
<DTPOSTED>20260415120000\r\n\
<TRNAMT>2500.00\r\n\
<FITID>20260415-0001\r\n\
<NAME>ACME EMPLOYER PAYROLL\r\n\
<MEMO>Bi-weekly direct deposit\r\n\
<STMTTRN>\r\n\
<TRNTYPE>DEBIT\r\n\
<DTPOSTED>20260418\r\n\
<TRNAMT>-42.18\r\n\
<FITID>20260418-0002\r\n\
<NAME>WHOLEFDS GRP #10403\r\n\
<MEMO>Groceries\r\n\
</BANKTRANLIST>\r\n\
</STMTRS>\r\n\
</STMTTRNRS>\r\n\
</BANKMSGSRSV1>\r\n\
</OFX>\r\n";

const XML_SAMPLE: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<?OFX OFXHEADER=\"200\" VERSION=\"200\" SECURITY=\"NONE\" OLDFILEUID=\"NONE\" NEWFILEUID=\"NONE\"?>\n\
<OFX>\n\
<BANKMSGSRSV1>\n\
<STMTTRNRS>\n\
<STMTRS>\n\
<BANKTRANLIST>\n\
<STMTTRN>\n\
<TRNTYPE>DEBIT</TRNTYPE>\n\
<DTPOSTED>20260420080000</DTPOSTED>\n\
<TRNAMT>-12.50</TRNAMT>\n\
<FITID>XML-001</FITID>\n\
<NAME>STARBUCKS  STORE 12345</NAME>\n\
<MEMO>Coffee</MEMO>\n\
</STMTTRN>\n\
<STMTTRN>\n\
<TRNTYPE>INT</TRNTYPE>\n\
<DTPOSTED>20260430</DTPOSTED>\n\
<TRNAMT>0.37</TRNAMT>\n\
<FITID>XML-002</FITID>\n\
<NAME>INTEREST PAID</NAME>\n\
</STMTTRN>\n\
</BANKTRANLIST>\n\
</STMTRS>\n\
</STMTTRNRS>\n\
</BANKMSGSRSV1>\n\
</OFX>\n";

#[test]
fn parses_ofx_1x_sgml_two_transactions() {
    let result = parse_ofx(SGML_SAMPLE).expect("SGML parse should succeed");
    assert_eq!(result.len(), 2, "expected 2 transactions in SGML sample");

    let income = &result[0];
    assert_eq!(income.fitid, "20260415-0001");
    assert_eq!(income.amount, Decimal::from_str("2500.00").unwrap());
    assert_eq!(income.date, NaiveDate::from_ymd_opt(2026, 4, 15).unwrap());
    assert_eq!(income.name.as_deref(), Some("ACME EMPLOYER PAYROLL"));
    assert_eq!(income.memo.as_deref(), Some("Bi-weekly direct deposit"));
    assert_eq!(income.trntype.as_deref(), Some("CREDIT"));

    let expense = &result[1];
    assert_eq!(expense.fitid, "20260418-0002");
    assert_eq!(expense.amount, Decimal::from_str("-42.18").unwrap());
    assert_eq!(expense.date, NaiveDate::from_ymd_opt(2026, 4, 18).unwrap());
    assert_eq!(expense.name.as_deref(), Some("WHOLEFDS GRP #10403"));
}

#[test]
fn parses_ofx_2x_xml_two_transactions() {
    let result = parse_ofx(XML_SAMPLE).expect("XML parse should succeed");
    assert_eq!(result.len(), 2, "expected 2 transactions in XML sample");
    assert_eq!(result[0].fitid, "XML-001");
    assert_eq!(result[0].amount, Decimal::from_str("-12.50").unwrap());
    assert_eq!(result[0].name.as_deref(), Some("STARBUCKS  STORE 12345"));
    assert_eq!(result[1].fitid, "XML-002");
    assert_eq!(result[1].amount, Decimal::from_str("0.37").unwrap());
    assert_eq!(result[1].memo, None);
}

#[test]
fn direction_inferred_from_amount_sign() {
    assert_eq!(
        direction_from_amount(Decimal::from_str("100.00").unwrap()),
        "INCOME"
    );
    assert_eq!(
        direction_from_amount(Decimal::from_str("-42.18").unwrap()),
        "EXPENSE"
    );
    // Zero is treated as INCOME — banks rarely emit 0.00 rows; sign is the
    // contract, and Decimal::ZERO is_sign_negative() is false.
    assert_eq!(direction_from_amount(Decimal::ZERO), "INCOME");
}

#[test]
fn empty_input_returns_empty_vec() {
    assert_eq!(parse_ofx("").unwrap(), Vec::<OfxTransaction>::new());
}

#[test]
fn skips_stmttrn_block_missing_fitid() {
    // STMTTRN without FITID is silently skipped (parser-level, not error).
    let body = "<OFX><BANKTRANLIST>\
<STMTTRN><DTPOSTED>20260415<TRNAMT>10.00<NAME>NO FITID HERE\
</BANKTRANLIST></OFX>";
    let result = parse_ofx(body).unwrap();
    assert!(result.is_empty(), "row without FITID must be skipped");
}

#[test]
fn malformed_amount_returns_validation_error() {
    let body = "<OFX><BANKTRANLIST>\
<STMTTRN>\
<FITID>BAD-AMOUNT\
<DTPOSTED>20260415\
<TRNAMT>not-a-number\
<NAME>BAD ROW\
</BANKTRANLIST></OFX>";
    let err = parse_ofx(body).unwrap_err().to_string();
    assert!(
        err.contains("invalid TRNAMT") && err.contains("not-a-number"),
        "expected TRNAMT parse error, got: {}",
        err
    );
}

#[test]
fn malformed_date_returns_validation_error() {
    let body = "<OFX><BANKTRANLIST>\
<STMTTRN>\
<FITID>BAD-DATE\
<DTPOSTED>20269999\
<TRNAMT>10.00\
<NAME>BAD DATE ROW\
</BANKTRANLIST></OFX>";
    let err = parse_ofx(body).unwrap_err().to_string();
    assert!(
        err.contains("invalid OFX date"),
        "expected OFX date parse error, got: {}",
        err
    );
}

#[test]
fn handles_payee_name_alternative_tag() {
    let body = "<OFX><BANKTRANLIST>\
<STMTTRN>\
<FITID>PAYEE-001\
<DTPOSTED>20260415\
<TRNAMT>50.00\
<PAYEE.NAME>ALT NAME TAG\
</BANKTRANLIST></OFX>";
    let result = parse_ofx(body).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name.as_deref(), Some("ALT NAME TAG"));
}

#[test]
fn long_date_string_consumes_only_yyyymmdd_prefix() {
    // Real OFX dates often have time + timezone: 20260415120000.000[-5:EST]
    let body = "<OFX><BANKTRANLIST>\
<STMTTRN>\
<FITID>WITH-TZ\
<DTPOSTED>20260415120000.000[-5:EST]\
<TRNAMT>10.00\
<NAME>TZ DATE ROW\
</BANKTRANLIST></OFX>";
    let result = parse_ofx(body).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(
        result[0].date,
        NaiveDate::from_ymd_opt(2026, 4, 15).unwrap()
    );
}
