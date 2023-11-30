use crate::chain::{ABISerializableObject, JSONValue};
use crate::serializer::encoder::ABIEncoder;

#[derive(Clone)]
pub struct PermissionLevel {
    actor: Name,
    permission: Name,
}

#[derive(Debug, Clone)]
pub enum PermissionLevelType {
    PermissionLevel(PermissionLevel),
    ActorPermission { actor: NameType, permission: NameType },
}

impl PermissionLevel {
    pub fn new(actor: Name, permission: Name) -> Self {
        PermissionLevel { actor, permission }
    }

    pub fn from(value: PermissionLevelType) -> Self {
        match value {
            PermissionLevelType::PermissionLevel(permission_level) => permission_level,
            PermissionLevelType::ActorPermission { actor, permission } => {
                let actor_name = Name::from(actor);
                let permission_name = Name::from(permission);
                PermissionLevel::new(actor_name, permission_name)
            }
        }
    }

    pub fn equals(&self, other: &PermissionLevelType) -> bool {
        if let PermissionLevelType::PermissionLevel(other_perm) = other {
            self.actor.equals(&other_perm.actor) && self.permission.equals(&other_perm.permission)
        } else {
            false
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}@{}", self.actor.to_string(), self.permission.to_string())
    }
}

impl ABISerializableObject for PermissionLevel {
    fn get_abi_name(&self) -> String {
        "permission_level".to_string()
    }

    fn to_abi(&self, encoder: &mut ABIEncoder) {
        self.actor.to_abi(encoder);
        self.permission.to_abi(encoder);
    }

    fn to_json(&self) -> JSONValue {
        JSONValue::String(self.to_string())
    }
}
