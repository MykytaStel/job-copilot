from fastapi import HTTPException, status


class WeeklyGuidanceProviderError(Exception):
    pass


class MalformedWeeklyGuidanceOutputError(WeeklyGuidanceProviderError):
    pass


def http_error_from_weekly_guidance_error(error: WeeklyGuidanceProviderError) -> HTTPException:
    if isinstance(error, MalformedWeeklyGuidanceOutputError):
        return HTTPException(
            status_code=status.HTTP_502_BAD_GATEWAY,
            detail="Weekly guidance provider returned malformed output.",
        )

    return HTTPException(
        status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
        detail=str(error) or "Weekly guidance provider failed.",
    )
