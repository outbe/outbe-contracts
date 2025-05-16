use cw20::Denom;

pub fn denom_to_string(denom: &Denom) -> String {
    match &denom {
        Denom::Native(value) => {
            let mut native = "native_".to_owned();
            native.push_str(value);
            native
        }
        Denom::Cw20(value) => {
            let mut cw20 = "cw20_".to_owned();
            cw20.push_str(value.as_str());
            cw20
        }
    }
}
