use crate::{
    error::RouterError,
    models::{Account, Organization, OrganizationName},
    DbPool,
};
use actix_web::web;
use diesel::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct OrgInfoUpdatebleFileds {
    username: String,
    name: String,
    profile_image: Option<String>,
    national_id: String,
}

/// Edits the org
pub async fn edit_organization(
    path: web::Path<u32>,
    info: web::Json<OrgInfoUpdatebleFileds>,
    pool: web::Data<DbPool>,
) -> Result<String, RouterError> {
    use crate::schema::app_accounts::dsl::{app_accounts, id as acc_id, username};
    use crate::schema::app_organization_names::dsl::{language, name};
    use crate::schema::app_organizations::dsl::*;

    let org_id = path.into_inner();
    let new_org = info.into_inner();

    let update_result: Result<String, RouterError> = web::block(move || {
        let mut conn = pool.get().unwrap();

        // First find the org from id
        let Ok(account) = app_accounts
            .filter(acc_id.eq(org_id as i32))
            .load::<Account>(&mut conn) else {
                return Err(RouterError::InternalError);
            };

        let Ok(org) = Organization::belonging_to(account.get(0).unwrap())
            .load::<Organization>(&mut conn)
            else {
                return Err(RouterError::InternalError)
            };

        let Some(account)= account.get(0) else {
            return Err(RouterError::NotFound("Account not found".to_string()));
        };

        let Some(org) = org.get(0) else {
            return Err(RouterError::NotFound("Organization not found".to_string()));
        };

        let Ok(_) = diesel::update(account)
            .set(username.eq(new_org.username))
            .execute(&mut conn) else {
                return Err(RouterError::InternalError);
            };

        let Ok(_) = diesel::update(&org)
            .set((
                profile_image.eq(new_org.profile_image),
                national_id.eq(new_org.national_id),
            ))
            .execute(&mut conn)
            else {
                return Err(RouterError::InternalError);
            };

        let Ok(org_name) = OrganizationName::belonging_to(account)
            // Get the primary name
            .filter(language.eq("default"))
            .load::<OrganizationName>(&mut conn) else {
                return Err(RouterError::NotFound("Organization not found".to_string()));
            };

        let Some(org_name) = org_name.get(0) else {
            return Err(RouterError::InternalError);
        };

        let Ok(_) = diesel::update(&org_name)
            .set((
                name.eq(new_org.name),
            ))
            .execute(&mut conn)
            else {
                return Err(RouterError::InternalError);
            };

        Ok("Updated".to_string())
    })
    .await
    .unwrap();

    update_result
}
