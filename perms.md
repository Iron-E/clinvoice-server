# Permissions

Bullet points note **exceptions** where someone who does not have explicit permission to perform an operation on a given resource may be given permission.

* Missing sections indicate that only `User`s with that appropriate permission should be able to perform the operation.

## Department

### GET (retrieve)

A `User` with an `Employee` record may retrieve information about their assigned `Department`.

## Employee

### GET (retrieve)

A `User` with an `Employee` record may retrieve information about themselves.

## Expenses

### DELETE (delete)

A `User` with an `Employee` record may delete `Expense`s attached to a `Timesheet` they have submitted.

### GET (retrieve)

A `User` with an `Employee` record may retrieve `Expense`s attached to a `Timesheet` they have submitted.

### PATCH (update)

A `User` with an `Employee` record may update `Expense`s attached to a `Timesheet` they have submitted.

### POST (create)

A `User` with an `Employee` record may create `Expense`s attached to a `Timesheet` they have submitted.

## Job

### DELETE (delete)

To perform this operation, two things must occur:

1. A given `User` must have permission to delete `Job`s, and
2. `User` must have an `Employee` record with a `Department` which is marked in-scope of the `Job`.

### GET (retrieve)

`User`s with an associated `Employee` can retrieve information about `Job`s in scope of their `Department`.

### PATCH (update)

To perform this operation, two things must occur:

1. A given `User` must have permission to update `Job`s, and
2. `User` must have an `Employee` record with a `Department` which is marked in-scope of the `Job`.

## Timesheet

### DELETE (delete)

A `User` with an `Employee` record may always delete `Timesheet`s they have submitted.

### GET (retrieve)

A `User` with an `Employee` record may always retrieve `Timesheet`s they have submitted.

### PATCH (update)

A `User` with an `Employee` record may always update `Timesheet`s they have submitted.

### POST (create)

A `User` with an `Employee` record may submit `Timesheet`s for `Job`s in-scope of their `Department`.

## User

### GET (retrieve)

A `User` can always retrieve information about themselves.

### PATCH (update)

A `User` can always update information about themselves.
