#[derive(Debug, Clone, Copy)]
pub enum Response<T, E> {
    Success(T),
    Rejection(E),
}
