//! calls to persistance storage
use uuid::Uuid;
use diesel::{
    insertable::Insertable,
    pg::{Pg, PgConnection},
    result::Error as DieselError,
    query_dsl::{self, filter_dsl::FindDsl, RunQueryDsl},
    query_builder::{AsQuery, InsertStatement, QueryFragment,  QueryId},
    query_source::Queryable,
    sql_types::HasSqlType,
};
use typename::TypeName;

use crate::error::{Error, ErrorKind};

pub fn handle_err<T: typename::TypeName>(error: DieselError) -> Error {
    match error {
        DieselError::NotFound => Error::from(ErrorKind::NotFound {
            type_name: T::type_name(),
        }),
        // Give some insight into what the internal state of the app is.
        // Set this to 'None' when the app enters into production stage.
        _ => Error::from(ErrorKind::DatabaseError(format!("Database error: {:?}", error))),
    }
}

/// Generic function for getting a whole row from a given table.
#[inline(always)]
pub fn get_row<'a, Model, Table>(table: Table, uuid: Uuid, conn: &PgConnection) -> Result<Model, Error>
where
    Table: FindDsl<Uuid>,
    diesel::dsl::Find<Table, Uuid>: query_dsl::LoadQuery<PgConnection, Model>,
    Model: typename::TypeName,
{
    table.find(uuid).get_result::<Model>(conn).map_err(handle_err::<Model>)
}

/// Generic function for creating a row for a given table with a given "new" struct for that row type.
#[inline(always)]
pub fn create_row<Model, NewModel, Tab>(table: Tab, insert: NewModel, conn: &PgConnection) -> Result<Model, Error>
where
    NewModel: diesel::insertable::Insertable<Tab>,
    InsertStatement<Tab, NewModel>: AsQuery,
    Pg: HasSqlType<<InsertStatement<Tab, NewModel> as AsQuery>::SqlType>,
    InsertStatement<Tab, <NewModel as Insertable<Tab>>::Values>: AsQuery,
    Model: Queryable<<InsertStatement<Tab, <NewModel as Insertable<Tab>>::Values> as AsQuery>::SqlType, Pg>,
    Pg: HasSqlType<<InsertStatement<Tab, <NewModel as Insertable<Tab>>::Values> as AsQuery>::SqlType>,
    <InsertStatement<Tab, <NewModel as Insertable<Tab>>::Values> as AsQuery>::Query: QueryId,
    <InsertStatement<Tab, <NewModel as Insertable<Tab>>::Values> as AsQuery>::Query: QueryFragment<Pg>,
    Model: TypeName,
{
    insert
        .insert_into(table)
        .get_result::<Model>(conn)
        .map_err(handle_err::<Model>)
}
