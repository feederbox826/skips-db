## Endpoints

### Open Endpoints
GET `/health`  
returns "ok"

GET `/api/time/{studio_id}`  
returns all submissions from the studio in the form of [AggregateStudio](#AggregateStudio)

HEAD `/api/time/{studio_id}`  
returns status code if submissions were found for studioID
- 202: found
- 404: no submissions
- 422: invalid uuid format

GET `/api/time/{studio_id}/submissions`  
returns all submissions from the studio in the form of [PublicSubmission](#PublicSubmission)

GET `/api/time/all`  
returns all submissions in the form of [PublicSubmission](#PublicSubmission)

### Authenticated
Authentication is done via Bearer `Authorization` with a UUID of any version. Header value should be in the following format: `Bearer 00000000-0000-0000-0000-000000000000`. See [Identifiers/ Privacy](#identifiers-privacy) for details

POST `/api/time/vote/{id}`  
```json
{
  "value": 1 // 1 for upvote, -1 for downvote
}
```

POST `/api/time/submit` 
```json
{
  "studio_id": "00000000-0000-0000-0000-000000000000", // uuid of studio
  "skip_seconds": 10 // seconds to skip from the start
}
```

POST `/api/user/name`  
```json
{
  "name": "user"
}
```

## Schema
### PublicSubmission
```json
{
  "id": 1, // id of submission
  "studio_id": "00000000-0000-0000-0000-000000000000", // uuid of studio
  "skip_seconds": 10, // seconds to skip from the start (0 = no intro / nothing to skip)
  "name": "anonymous", // name of the user if they chose to identify themselves
  "net_votes": 2 // sum of votes for or against segment
}
```
### AggregateStudio
```json
{
  "studio_id": "00000000-0000-0000-0000-000000000000", // uuid of studio
  "skip_seconds": 10 // seconds to skip from the start (0 = no intro / nothing to skip, max 60)
}
```

## Identifiers/ Privacy
Users are auth'd with a random UUID. This is associated with all of your submissions and votes as an anti-spam measure. Users can optionally choose to identify themselves by associating an alias with their UUID. This can be changed at any time to revert the username displayed to "anonymous"

## Sorting/ Order
- highest votes
- lowest time