#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Direction {
    Forward = 0,
    Backward = 1,
}


#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
/// Used to specify which actuator a command is meant for.
pub enum Actuator {
    /// the lift
    M1 = 0,
    /// the bucket
    M2 = 1
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActuatorCommand {
    SetSpeed(u16, Actuator),
    SetDirection(Direction, Actuator),
}

impl ActuatorCommand {
    pub fn deserialize(bytes: [u8; 4]) -> Result<Self, &'static str> {
        let actuator = {
            if bytes[3] == Actuator::M1 as u8 {
                Actuator::M1
            } else if bytes[3] == Actuator::M2 as u8{
                Actuator::M2
            } else {
                return Err("Unknown actuator specifier (not m1 or m2)");
            }
        };
        match bytes[0] {
            tag if tag == 0 => {
                let speed = u16::from_le_bytes(
                    bytes[1..=2]
                        .try_into()
                        .map_err(|_| "Wrong number of bytes in actuator command")?,
                );
                Ok(ActuatorCommand::SetSpeed(speed, actuator))
            }
            tag if tag == 1 => {
                let dir = match bytes[1] {
                    0 => Direction::Forward,
                    1 => Direction::Backward,
                    _ => return Err("Invalid direction value"),
                };
                Ok(ActuatorCommand::SetDirection(dir, actuator))
            }
            _ => Err("Invalid variant tag"),
        }
    }

    pub fn serialize(&self) -> [u8; 4] {
        match self {
            ActuatorCommand::SetSpeed(speed, actuator) => {
                let mut bytes = [0u8; 4];
                bytes[0] = 0;
                bytes[1..=2].copy_from_slice(&speed.to_le_bytes());
                bytes[3] = *actuator as u8;
                bytes
            }
            ActuatorCommand::SetDirection(dir, actuator) => {
                let mut bytes = [0u8; 4];
                bytes[0] = 1;
                bytes[1] = *dir as u8;
                bytes[2] = 0;
                bytes[3] = *actuator as u8;
                bytes
            }
        }
    }
}