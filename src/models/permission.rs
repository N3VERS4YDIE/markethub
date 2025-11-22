use std::collections::{BTreeMap, BTreeSet};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Permission {
    // Products
    ViewProducts,
    CreateProducts,
    EditProducts,
    DeleteProducts,
    // Orders
    ViewOrders,
    ProcessOrders,
    CancelOrders,
    // Members
    ViewMembers,
    InviteMembers,
    EditPermissions,
    // Access
    GrantAccess,
    RevokeAccess,
    // Analytics
    ViewStats,
    ExportReports,
}

impl Permission {
    pub fn as_str(&self) -> &'static str {
        match self {
            Permission::ViewProducts => "VIEW_PRODUCTS",
            Permission::CreateProducts => "CREATE_PRODUCTS",
            Permission::EditProducts => "EDIT_PRODUCTS",
            Permission::DeleteProducts => "DELETE_PRODUCTS",
            Permission::ViewOrders => "VIEW_ORDERS",
            Permission::ProcessOrders => "PROCESS_ORDERS",
            Permission::CancelOrders => "CANCEL_ORDERS",
            Permission::ViewMembers => "VIEW_MEMBERS",
            Permission::InviteMembers => "INVITE_MEMBERS",
            Permission::EditPermissions => "EDIT_PERMISSIONS",
            Permission::GrantAccess => "GRANT_ACCESS",
            Permission::RevokeAccess => "REVOKE_ACCESS",
            Permission::ViewStats => "VIEW_STATS",
            Permission::ExportReports => "EXPORT_REPORTS",
        }
    }

    pub fn all() -> &'static [Permission] {
        &PERMISSION_LIST
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub static PERMISSION_LIST: [Permission; 14] = [
    Permission::ViewProducts,
    Permission::CreateProducts,
    Permission::EditProducts,
    Permission::DeleteProducts,
    Permission::ViewOrders,
    Permission::ProcessOrders,
    Permission::CancelOrders,
    Permission::ViewMembers,
    Permission::InviteMembers,
    Permission::EditPermissions,
    Permission::GrantAccess,
    Permission::RevokeAccess,
    Permission::ViewStats,
    Permission::ExportReports,
];

pub static ROLE_PERMISSIONS: Lazy<BTreeMap<&'static str, BTreeSet<Permission>>> = Lazy::new(|| {
    use Permission::*;
    let mut map = BTreeMap::new();

    map.insert(
        "Owner",
        PERMISSION_LIST.into_iter().collect::<BTreeSet<_>>(),
    );

    map.insert(
        "Admin",
        [
            ViewProducts,
            CreateProducts,
            EditProducts,
            DeleteProducts,
            ViewOrders,
            ProcessOrders,
            CancelOrders,
            ViewMembers,
            InviteMembers,
            EditPermissions,
            GrantAccess,
            RevokeAccess,
            ViewStats,
            ExportReports,
        ]
        .into_iter()
        .collect(),
    );

    map.insert(
        "Manager",
        [
            ViewProducts,
            CreateProducts,
            EditProducts,
            ViewOrders,
            ProcessOrders,
            ViewStats,
        ]
        .into_iter()
        .collect(),
    );

    map.insert("Staff", [ViewProducts, ViewOrders].into_iter().collect());

    map
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_has_all_permissions() {
        let owner = ROLE_PERMISSIONS.get("Owner").unwrap();
        assert_eq!(owner.len(), PERMISSION_LIST.len());
    }
}
