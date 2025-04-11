#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Direction {
    Forward = 0,
    Backward = 1,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActuatorCommand {
    SetSpeed(u16),
    SetDirection(Direction),
}

impl ActuatorCommand {
    pub fn serialize(&self) -> [u8; 3] {
        match self {
            ActuatorCommand::SetSpeed(speed) => {
                let mut bytes = [0u8; 3];
                bytes[0] = 0;
                bytes[1..].copy_from_slice(&speed.to_le_bytes());
                bytes
            }
            ActuatorCommand::SetDirection(dir) => {
                let mut bytes = [0u8; 3];
                bytes[0] = 1;
                bytes[1] = *dir as u8;
                bytes[2] = 0;
                bytes
            }
        }
    }

    pub fn set_speed(mut speed: f64) -> Self {
        speed = speed.clamp(0.0, 1.0);
        ActuatorCommand::SetSpeed((speed * u16::MAX as f64) as u16)
    }

    pub fn forward() -> Self {
        ActuatorCommand::SetDirection(Direction::Forward)
    }

    pub fn backward() -> Self {
        ActuatorCommand::SetDirection(Direction::Backward)
    }
}