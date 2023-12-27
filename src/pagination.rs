use crate::{db::DbConnection, error::bad_request, types::AppResult};
use diesel::{
    pg::Pg, prelude::*, query_builder::*, query_dsl::methods::LoadQuery, sql_types::BigInt,
};
use serde::{Deserialize, Serialize};

// https://github.com/rust-lang/crates.io/blob/61c08708bdc04c081283a6880cf53834880b8409/src/controllers/keyword.rs#L28
//
//https://github.com/rust-lang/crates.io/blob/main/src/controllers/keyword.rs#L28
//
//https://github.com/rust-lang/crates.io/blob/83b70cfb0b7cd262109d0e245bef5534916449cb/src/controllers/user/me.rs#L76

const DEFAULT_PER_PAGE: i64 = 10;
const MAX_PER_PAGE: i64 = 25;

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub(crate) enum Page {
//     Numeric(i64),
//     // Unspecified,
// }
//
//
#[derive(Serialize)]
pub struct PaginationMeta {
    pub total: i64,
}
#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64) -> Self {
        Self {
            data,
            meta: PaginationMeta { total },
        }
    }
}

impl<T> From<Paginated<T>> for PaginatedResponse<T>
where
    T: std::clone::Clone,
{
    fn from(value: Paginated<T>) -> Self {
        let data = value.iter().cloned().collect();
        Self {
            data,
            meta: PaginationMeta {
                total: value.total(),
            },
        }
    }
}

/// Used in the query extractor to pull pagination
#[derive(Deserialize, Debug)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Clone)]
pub(crate) struct PaginationOptions {
    pub(crate) page: i64,
    pub(crate) per_page: i64,
}

impl PaginationOptions {
    pub(crate) fn new(params: PaginationParams) -> AppResult<PaginationOptions> {
        if params
            .per_page
            .is_some_and(|x| !(1..=MAX_PER_PAGE).contains(&x))
        {
            return Err(bad_request(
                "Invalid per_page query: min=1, max={MAX_PER_PAGE}",
            ));
        }

        if params.page.is_some_and(|x| x < 1) {
            return Err(bad_request("Invalid page query: min=1"));
        }

        Ok(PaginationOptions {
            page: params.page.unwrap_or(1),
            per_page: params.per_page.unwrap_or(DEFAULT_PER_PAGE),
        })
    }

    pub(crate) fn offset(&self) -> Option<i64> {
        let pp = self.per_page;
        Some((self.page - 1) * pp)
    }
}

pub(crate) trait Paginate: Sized {
    fn pages_pagination(self, options: PaginationOptions) -> PaginatedQuery<Self> {
        PaginatedQuery {
            query: self,
            options,
        }
    }

    // fn pages_pagination_with_count_query<C>(
    //     self,
    //     options: PaginationOptions,
    //     count_query: C,
    // ) -> PaginatedQueryWithCountSubq<Self, C> {
    //     PaginatedQueryWithCountSubq {
    //         query: self,
    //         count_query,
    //         options,
    //     }
    // }
}

impl<T> Paginate for T {}

pub struct Paginated<T> {
    records_and_total: Vec<WithCount<T>>,
    // options: PaginationOptions,
}

impl<T> Paginated<T> {
    pub(crate) fn total(&self) -> i64 {
        self.records_and_total
            .first()
            .map(|row| row.total)
            .unwrap_or_default() // If there is no first row, then the total is zero.
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> {
        self.records_and_total.iter().map(|row| &row.record)
    }
}

impl<T: 'static> IntoIterator for Paginated<T> {
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.records_and_total.into_iter().map(|row| row.record))
    }
}
#[derive(Debug)]
pub(crate) struct PaginatedQuery<T> {
    query: T,
    options: PaginationOptions,
}
impl<T> PaginatedQuery<T> {
    pub(crate) fn load<'a, U>(self, conn: &mut DbConnection) -> QueryResult<Paginated<U>>
    where
        Self: LoadQuery<'a, DbConnection, WithCount<U>>,
    {
        // let options = self.options.clone();
        let records_and_total = self.internal_load(conn)?.collect::<QueryResult<_>>()?;
        Ok(Paginated {
            records_and_total,
            // options,
        })
    }
}

impl<T> QueryId for PaginatedQuery<T> {
    const HAS_STATIC_QUERY_ID: bool = false;
    type QueryId = ();
}

impl<T: Query> Query for PaginatedQuery<T> {
    type SqlType = (T::SqlType, BigInt);
}

impl<T, DB> RunQueryDsl<DB> for PaginatedQuery<T> {}

impl<T> QueryFragment<Pg> for PaginatedQuery<T>
where
    T: QueryFragment<Pg>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> QueryResult<()> {
        out.push_sql("SELECT *, COUNT(*) OVER () FROM (");
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(") t LIMIT ");
        out.push_bind_param::<BigInt, _>(&self.options.per_page)?;
        if let Some(offset) = self.options.offset() {
            out.push_sql(format!(" OFFSET {offset}").as_str());
        }
        Ok(())
    }
}

// #[derive(Debug)]
// pub(crate) struct PaginatedQueryWithCountSubq<T, C> {
//     query: T,
//     count_query: C,
//     options: PaginationOptions,
// }

// impl<T, C> QueryId for PaginatedQueryWithCountSubq<T, C> {
//     const HAS_STATIC_QUERY_ID: bool = false;
//     type QueryId = ();
// }

// impl<
//         T: Query,
//         C: Query + QueryDsl + diesel::query_dsl::methods::SelectDsl<diesel::dsl::CountStar>,
//     > Query for PaginatedQueryWithCountSubq<T, C>
// {
//     type SqlType = (T::SqlType, BigInt);
// }

// impl<T, C, DB> RunQueryDsl<DB> for PaginatedQueryWithCountSubq<T, C> {}

// impl<T, C> QueryFragment<Pg> for PaginatedQueryWithCountSubq<T, C>
// where
//     T: QueryFragment<Pg>,
//     C: QueryFragment<Pg>,
// {
//     fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> QueryResult<()> {
//         out.push_sql("SELECT *, (");
//         self.count_query.walk_ast(out.reborrow())?;
//         out.push_sql(") FROM (");
//         self.query.walk_ast(out.reborrow())?;
//         out.push_sql(") t LIMIT ");
//         out.push_bind_param::<BigInt, _>(&self.options.per_page)?;
//         if let Some(offset) = self.options.offset() {
//             // Injection safety: `offset()` returns `Option<i64>`, so this interpolation is
//             // constrained to known valid values and this is not vulnerable to user
//             // injection attacks.
//             out.push_sql(format!(" OFFSET {offset}").as_str());
//         }
//         Ok(())
//     }
// }

// impl<T, C> PaginatedQueryWithCountSubq<T, C> {
//     pub(crate) fn load<'a, U>(self, conn: &mut DbConnection) -> QueryResult<Paginated<U>>
//     where
//         Self: LoadQuery<'a, DbConnection, WithCount<U>>,
//     {
//         let options = self.options.clone();
//         let records_and_total = self.internal_load(conn)?.collect::<QueryResult<_>>()?;
//         Ok(Paginated {
//             records_and_total,
//             options,
//         })
//     }
// }
#[derive(QueryableByName, Queryable, Debug)]
pub struct WithCount<T> {
    #[diesel(embed)]
    pub(crate) record: T,
    #[diesel(sql_type = BigInt)]
    pub(crate) total: i64,
}

pub trait WithCountExtension<T> {
    fn records_and_total(self) -> (Vec<T>, i64);
}

impl<T> WithCountExtension<T> for Vec<WithCount<T>> {
    fn records_and_total(self) -> (Vec<T>, i64) {
        let cnt = self.first().map(|row| row.total).unwrap_or(0);
        let vec = self.into_iter().map(|row| row.record).collect();
        (vec, cnt)
    }
}
