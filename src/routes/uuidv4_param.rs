use warp::Filter;
use warp::Rejection;
use uuid::Uuid;
use crate::error::Error;

pub fn uuidv4_param() -> impl Filter<Extract = (Uuid,), Error = Rejection> + Copy {
    warp::path::param()
        .and_then(|id: String| async move {
            Uuid::parse_str(&id)
                .map_err(|_| warp::reject::custom(Error::InvalidId))
        })
}
