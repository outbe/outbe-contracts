use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use std::fmt::{Display, Formatter};

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

/// Currency code in ISO 4217 format. Please see for details
/// [wiki](https://en.wikipedia.org/wiki/ISO_4217)
#[cw_serde]
#[derive(Copy, Eq)]
pub enum Currency {
    Usd,
    Eur,
    // todo add others when required
}

/// Commodity type representing precious metals and other commodities
#[cw_serde]
#[derive(Copy, Eq)]
pub enum CommodityType {
    Xau, // gold
}

impl Display for Currency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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

    #[test]
    fn test_denom_native_display() {
        let denom = Denom::Native("ujuno".to_string());
        assert_eq!(denom.to_string(), "native_ujuno");
    }

    #[test]
    fn test_denom_cw20_display() {
        let deps = mock_dependencies();
        let addr = Addr::unchecked("cosmos2contract");
        let denom = Denom::Cw20(addr.clone());
        assert_eq!(denom.to_string(), format!("cw20_{}", addr));
    }

    #[test]
    fn test_denom_fiat_display() {
        let denom = Denom::Fiat(Currency::Usd);
        assert_eq!(denom.to_string(), "fiat_Usd");

        let denom = Denom::Fiat(Currency::Eur);
        assert_eq!(denom.to_string(), "fiat_Eur");
    }

    #[test]
    fn test_currency_display() {
        let currency = Currency::Usd;
        assert_eq!(currency.to_string(), "Usd");

        let currency = Currency::Eur;
        assert_eq!(currency.to_string(), "Eur");
    }

    #[test]
    fn test_commodity_type_display() {
        let commodity = CommodityType::Xau;
        assert_eq!(commodity.to_string(), "Xau");
    }
}
