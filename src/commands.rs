#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Direction {
    Forward = 0,
    Backward = 1,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum ActuatorCommand {
    SetSpeed(u16) = 0,
    SetDirection(Direction) = 1,
}

impl ActuatorCommand {
    pub fn deserialize(bytes: [u8; 3]) -> Result<Self, &'static str> {
        match bytes[0] {
            tag if tag == ActuatorCommand::SetSpeed as u8 => {
                let speed = u16::from_le_bytes(
                    bytes[1..]
                        .try_into()
                        .map_err(|_| "Wrong number of bytes in actuator command")?,
                );
                Ok(ActuatorCommand::SetSpeed(speed))
            }
            tag if tag == ActuatorCommand::SetDirection as u8 => {
                let dir = match bytes[1] {
                    0 => Direction::Forward,
                    1 => Direction::Backward,
                    _ => return Err("Invalid direction value"),
                };
                Ok(ActuatorCommand::SetDirection(dir))
            }
            _ => Err("Invalid variant tag"),
        }
    }

    pub fn serialize(&self) -> [u8; 3] {
        match self {
            ActuatorCommand::SetSpeed(speed) => {
                let mut bytes = [0u8; 3];
                bytes[0] = ActuatorCommand::SetSpeed as u8;
                bytes[1..].copy_from_slice(&speed.to_le_bytes());
                bytes
            }
            ActuatorCommand::SetDirection(dir) => {
                let mut bytes = [0u8; 3];
                bytes[0] = ActuatorCommand::SetDirection as u8;
                bytes[1] = *dir as u8;
                bytes[2] = 0;
                bytes
            }
        }
    }
}