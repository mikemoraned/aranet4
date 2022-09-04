use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Sample {
    pub co2: u16,
    pub temp: f32,
    pub pressure: f32,
    pub humidity: u8,
    pub battery: u8
}

impl TryFrom<&Vec<u8>> for Sample {
    type Error = String;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {

        let minimum_length = 8;
        if value.len() < minimum_length {
            return Err(format!("response is too short; needs to have at least {} elements", minimum_length));
        }

        let sample = Sample {
            co2 : u16::from_le_bytes([ value[0], value[1] ]),
            temp : u16::from_le_bytes([ value[2], value[3] ]) as f32 / 20.0,
            pressure : u16::from_le_bytes([ value[4], value[5] ]) as f32 / 10.0,
            humidity : value[6],
            battery : value[7],
        };
        Ok(sample)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_with_good_full_response() {
        let response: Vec<u8> = vec![164, 5, 238, 1, 60, 39, 60, 98, 3];
        let expected = Sample { 
            co2: 1444, temp: 24.7, pressure: 1004.4, humidity: 60, battery: 98 
        };
        let actual 
            = Sample::try_from(&response).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_with_good_minimal_response() {
        let response: Vec<u8> = vec![164, 5, 238, 1, 60, 39, 60, 98];
        let expected = Sample { 
            co2: 1444, temp: 24.7, pressure: 1004.4, humidity: 60, battery: 98 
        };
        let actual 
            = Sample::try_from(&response).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_with_too_short_response() {
        let response: Vec<u8> = vec![164, 5, 238, 1, 60, 39, 60];
        let expected 
            = Result::Err("response is too short; needs to have at least 8 elements".into());
        let actual 
            = Sample::try_from(&response);
        assert_eq!(expected, actual);
    }
}