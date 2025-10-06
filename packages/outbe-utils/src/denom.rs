use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use std::fmt::{Display, Formatter};
use thiserror::Error;

/// Denom type represents a native currency, token or fiat
#[cw_serde]
pub enum Denom {
    Native(String),
    Cw20(Addr),
    Fiat(Currency),
    Commodity(CommodityType),
}

impl Display for Denom {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Native(value) => {
                write!(f, "native_{}", value)
            }
            Self::Cw20(value) => {
                write!(f, "cw20_{}", value)
            }
            Self::Fiat(value) => {
                write!(f, "fiat_{}", value)
            }
            Self::Commodity(value) => {
                write!(f, "commodity_{}", value)
            }
        }
    }
}

/// Numeric currency code
type CurrencyCode = u16;

/// Currency code in ISO 4217 format. Please see for details
/// [wiki](https://en.wikipedia.org/wiki/ISO_4217)
#[cw_serde]
#[derive(Copy, Eq)]
pub enum Currency {
    Usd = 840, // US Dollar
    Eur = 978, // Euro
    Gbp = 826, // British Pound Sterling
    Jpy = 392, // Japanese Yen
    Chf = 756, // Swiss Franc
    Cad = 124, // Canadian Dollar
    Aud = 36,  // Australian Dollar
    Cny = 156, // Chinese Yuan
    Inr = 356, // Indian Rupee
    Krw = 410, // South Korean Won
    Brl = 986, // Brazilian Real
    Mxn = 484, // Mexican Peso
    Sgd = 702, // Singapore Dollar
    Hkd = 344, // Hong Kong Dollar
    Nok = 578, // Norwegian Krone
    Sek = 752, // Swedish Krona
    Dkk = 208, // Danish Krone
    Pln = 985, // Polish Zloty
    Try = 949, // Turkish Lira
    Zar = 710, // South African Rand
    Thb = 764, // Thai Baht
    Nzd = 554, // New Zealand Dollar
}

impl Currency {
    /// Returns the numeric ISO 4217 code
    pub fn numeric_code(&self) -> u16 {
        *self as u16
    }

    /// Returns the alphabetic ISO 4217 code
    pub fn alpha_code(&self) -> &'static str {
        match self {
            Currency::Usd => "USD",
            Currency::Eur => "EUR",
            Currency::Gbp => "GBP",
            Currency::Jpy => "JPY",
            Currency::Chf => "CHF",
            Currency::Cad => "CAD",
            Currency::Aud => "AUD",
            Currency::Cny => "CNY",
            Currency::Inr => "INR",
            Currency::Krw => "KRW",
            Currency::Brl => "BRL",
            Currency::Mxn => "MXN",
            Currency::Sgd => "SGD",
            Currency::Hkd => "HKD",
            Currency::Nok => "NOK",
            Currency::Sek => "SEK",
            Currency::Dkk => "DKK",
            Currency::Pln => "PLN",
            Currency::Try => "TRY",
            Currency::Zar => "ZAR",
            Currency::Thb => "THB",
            Currency::Nzd => "NZD",
        }
    }

    /// Returns the standard minor units (decimal places)
    pub fn minor_units(&self) -> u8 {
        match self {
            Currency::Jpy | Currency::Krw => 0, // No decimal places
            // Most currencies use 2 decimal places
            _ => 2,
        }
    }
}

impl From<Currency> for CurrencyCode {
    fn from(val: Currency) -> Self {
        val.numeric_code()
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum CurrencyError {
    #[error("Wrong Currency Code: {0}")]
    WrongCode(CurrencyCode),
}

impl TryFrom<CurrencyCode> for Currency {
    type Error = CurrencyError;

    fn try_from(value: CurrencyCode) -> Result<Self, Self::Error> {
        use Currency::*;
        match value {
            840 => Ok(Usd),
            978 => Ok(Eur),
            826 => Ok(Gbp),
            392 => Ok(Jpy),
            756 => Ok(Chf),
            124 => Ok(Cad),
            36 => Ok(Aud),
            156 => Ok(Cny),
            356 => Ok(Inr),
            410 => Ok(Krw),
            986 => Ok(Brl),
            484 => Ok(Mxn),
            702 => Ok(Sgd),
            344 => Ok(Hkd),
            578 => Ok(Nok),
            752 => Ok(Sek),
            208 => Ok(Dkk),
            985 => Ok(Pln),
            949 => Ok(Try),
            710 => Ok(Zar),
            764 => Ok(Thb),
            554 => Ok(Nzd),
            _ => Err(CurrencyError::WrongCode(value)),
        }
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.alpha_code())
    }
}

/// Commodity type representing precious metals and other commodities
#[cw_serde]
#[derive(Copy, Eq)]
pub enum CommodityType {
    Xau, // gold
}

impl Display for CommodityType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::to_json_string;

    #[test]
    fn test_currency_numeric_codes() {
        assert_eq!(Currency::Usd.numeric_code(), 840);
        assert_eq!(Currency::Eur.numeric_code(), 978);
        assert_eq!(Currency::Gbp.numeric_code(), 826);
        assert_eq!(Currency::Jpy.numeric_code(), 392);
    }

    #[test]
    fn test_currency_serde() {
        #[cw_serde]
        struct Test {
            currency: Currency,
        }

        let json = to_json_string(&Test {
            currency: Currency::Usd,
        })
        .unwrap();

        println!("{}", json);
    }

    #[test]
    fn test_currency_alpha_codes() {
        assert_eq!(Currency::Usd.alpha_code(), "USD");
        assert_eq!(Currency::Eur.alpha_code(), "EUR");
        assert_eq!(Currency::Jpy.alpha_code(), "JPY");
    }

    #[test]
    fn test_currency_display() {
        assert_eq!(Currency::Usd.to_string(), "USD");
        assert_eq!(Currency::Eur.to_string(), "EUR");
    }

    #[test]
    fn test_minor_units() {
        assert_eq!(Currency::Usd.minor_units(), 2);
        assert_eq!(Currency::Jpy.minor_units(), 0); // Yen has no decimal places
        assert_eq!(Currency::Krw.minor_units(), 0); // Won has no decimal places
    }

    #[test]
    fn test_currency_equality() {
        let usd1 = Currency::Usd;
        let usd2 = Currency::Usd;
        let eur = Currency::Eur;

        assert_eq!(usd1, usd2);
        assert_ne!(usd1, eur);
    }

    #[test]
    fn test_denom_native_display() {
        let denom = Denom::Native("ujuno".to_string());
        assert_eq!(denom.to_string(), "native_ujuno");
    }

    #[test]
    fn test_denom_cw20_display() {
        let _deps = mock_dependencies();
        let addr = Addr::unchecked("cosmos2contract");
        let denom = Denom::Cw20(addr.clone());
        assert_eq!(denom.to_string(), format!("cw20_{}", addr));
    }

    #[test]
    fn test_commodity_type_display() {
        let commodity = CommodityType::Xau;
        assert_eq!(commodity.to_string(), "Xau");
    }
}
