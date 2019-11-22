use diesel::{self, prelude::*};
use diesel::result::Error;
use itertools::Itertools;
use rcir;

mod schema {
    
    table! {
        users (id) {
            id -> Int4,
            username -> Text,
        }
    }
    
    table! {
        items (id) {
            id -> Int4,
            title -> Text,
            body -> Text,
            done -> Bool,
        }
    }
    
    table! {
        votes (user_id, item_id) {
            user_id -> Int4,
            item_id -> Int4,
            ordinal -> Int4,
        }
    }
    
    joinable!(votes -> items (item_id));
    joinable!(votes -> users (user_id));
    allow_tables_to_appear_in_same_query!(items, users, votes);
    
}

use self::schema::users;
use self::schema::votes;

#[derive(Queryable, Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
}

#[derive(Serialize, Queryable, Debug)]
pub struct Item {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub done: bool,
}

#[table_name = "votes"]
#[derive(Queryable, Insertable, Debug, Clone)]
pub struct Vote {
    pub user_id: i32,
    pub item_id: i32,
    pub ordinal: i32,
}

use self::schema::items::dsl::{done as item_done, items as all_items};
use self::schema::users::dsl::{username as users_uname, users as all_users};
use self::schema::votes::dsl::{item_id, ordinal, user_id, votes as all_votes};

#[derive(Deserialize)]
pub struct Ballot {
    pub votes: Vec<i32>,
}

#[table_name = "users"]
#[derive(FromForm, Insertable)]
pub struct NewUser {
    pub username: String,
}

impl NewUser {
    pub fn login(self, conn: &PgConnection) -> Result<User, Error> {
        // ensure that the user exists
        let _ = diesel::insert_into(self::schema::users::table)
            .values(&self)
            .execute(conn);

        all_users
            .filter(users_uname.eq(&self.username))
            .get_result::<User>(conn)
    }
}

impl Item {
    pub fn for_user(uid: i32, conn: &PgConnection) -> Vec<(Item, Option<i32>)> {
        all_items
            .left_join(
                self::schema::votes::table
                    .on(user_id.eq(&uid).and(item_id.eq(self::schema::items::id))),
            )
            .filter(self::schema::items::done.eq(false))
            .order((user_id.desc(), ordinal.asc()))
            .select((self::schema::items::all_columns, ordinal.nullable()))
            .load::<(Item, Option<i32>)>(conn)
            .unwrap()
    }
}

impl Vote {
    pub fn run_election(conn: &PgConnection) -> Option<Item> {
        let votes = all_votes
            .inner_join(self::schema::items::table)
            .filter(item_done.eq(false))
            .order((user_id.asc(), ordinal.asc()))
            .select((user_id, item_id, ordinal))
            .get_results::<Vote>(conn)
            .unwrap();

        // the extra collections here are sad.
        let votes: Vec<Vec<_>> = votes
            .into_iter()
            .group_by(|v| v.user_id)
            .into_iter()
            .map(|(_, ballot)| ballot.into_iter().map(|v| v.item_id).collect())
            .collect();

        match rcir::run_election(&votes, rcir::MajorityMode::RemainingMajority).ok()? {
            rcir::ElectionResult::Winner(&iid) => {
                Some(all_items.find(iid).get_result::<Item>(conn).unwrap())
            }
            rcir::ElectionResult::Tie(iids) => {
                // TODO: maybe pick the oldest one?
                Some(all_items.find(*iids[0]).get_result::<Item>(conn).unwrap())
            }
        }
    }

    pub fn run_second_election(conn: &PgConnection, winner: &Option<Item>) -> Option<Item> {
        let winner = winner.as_ref()?;

        let votes = all_votes
            .inner_join(self::schema::items::table)
            .filter(item_done.eq(false))
            .filter(item_id.ne(winner.id))
            .order((user_id.asc(), ordinal.asc()))
            .select((user_id, item_id, ordinal))
            .get_results::<Vote>(conn)
            .unwrap();

        // the extra collections here are sad.
        let votes: Vec<Vec<_>> = votes
            .into_iter()
            .group_by(|v| v.user_id)
            .into_iter()
            .map(|(_, ballot)| ballot.into_iter().map(|v| v.item_id).collect())
            .collect();

        match rcir::run_election(&votes, rcir::MajorityMode::RemainingMajority).ok()? {
            rcir::ElectionResult::Winner(&iid) => {
                Some(all_items.find(iid).get_result::<Item>(conn).unwrap())
            }
            rcir::ElectionResult::Tie(iids) => {
                // TODO: maybe pick the oldest one?
                Some(all_items.find(*iids[0]).get_result::<Item>(conn).unwrap())
            }
        }
    }

    pub fn save_ballot(uid: i32, ballot: Ballot, conn: &PgConnection) {
        diesel::delete(all_votes.filter(user_id.eq(&uid)))
            .execute(conn)
            .unwrap();

        for (i, iid) in ballot.votes.into_iter().enumerate() {
            diesel::insert_into(self::schema::votes::table)
                .values(Vote {
                    user_id: uid,
                    item_id: iid,
                    ordinal: i as i32,
                })
                .execute(conn)
                .unwrap();
        }
    }
}
