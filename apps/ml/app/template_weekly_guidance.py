from typing import Any

from app.enrichment.weekly_guidance.contract import WeeklyGuidanceRequest
from app.enrichment.weekly_guidance.prompt import WeeklyGuidancePrompt


def build_weekly_guidance(
    context: WeeklyGuidanceRequest, prompt: WeeklyGuidancePrompt
) -> dict[str, Any]:
    analytics = context.analytics_summary
    behavior = context.behavior_summary
    funnel = context.funnel_summary
    llm_context = context.llm_context

    primary_role = (
        llm_context.analyzed_profile.primary_role.replace("_", " ")
        if llm_context.analyzed_profile
        else analytics.top_matched_roles[0].replace("_", " ")
        if analytics.top_matched_roles
        else "current target roles"
    )

    positive_source = behavior.top_positive_sources[0] if behavior.top_positive_sources else None
    negative_source = behavior.top_negative_sources[0] if behavior.top_negative_sources else None
    positive_role = (
        behavior.top_positive_role_families[0] if behavior.top_positive_role_families else None
    )
    negative_role = (
        behavior.top_negative_role_families[0] if behavior.top_negative_role_families else None
    )

    what_is_working: list[str] = []
    if positive_source and positive_source.net_score > 0:
        what_is_working.append(
            f"{positive_source.key} shows the strongest positive source signal with {positive_source.save_count} saves and {positive_source.application_created_count} applications."
        )
    if positive_role and positive_role.net_score > 0:
        what_is_working.append(
            f"{positive_role.key} role-family behavior is positive, with {positive_role.save_count} saves and {positive_role.application_created_count} applications."
        )
    if funnel.save_count > 0:
        what_is_working.append(
            f"The funnel is converting some interest into saves: {funnel.save_count} saves from {funnel.open_count} opens."
        )
    if analytics.feedback.saved_jobs_count > analytics.feedback.bad_fit_jobs_count:
        what_is_working.append(
            f"Saved jobs currently outnumber bad-fit jobs ({analytics.feedback.saved_jobs_count} vs {analytics.feedback.bad_fit_jobs_count})."
        )
    if not what_is_working:
        what_is_working.append("Evidence is still limited, but the current data is enough to keep testing focused search iterations.")

    what_is_not_working: list[str] = []
    if negative_source and negative_source.net_score < 0:
        what_is_not_working.append(
            f"{negative_source.key} is producing more negative than positive signals, with {negative_source.hide_count} hides and {negative_source.bad_fit_count} bad fits."
        )
    if negative_role and negative_role.net_score < 0:
        what_is_not_working.append(
            f"{negative_role.key} role-family behavior is negative, with {negative_role.hide_count} hides and {negative_role.bad_fit_count} bad fits."
        )
    if funnel.hide_count > funnel.save_count and funnel.hide_count > 0:
        what_is_not_working.append(
            f"Hides are still high relative to saves ({funnel.hide_count} vs {funnel.save_count}), which suggests search precision is weak."
        )
    if funnel.application_created_count == 0 and funnel.save_count > 0:
        what_is_not_working.append(
            f"Saved jobs are not converting into applications yet ({funnel.save_count} saves, 0 applications)."
        )
    elif funnel.application_created_count < funnel.save_count:
        what_is_not_working.append(
            f"Application volume is lagging behind saves ({funnel.application_created_count} applications from {funnel.save_count} saves)."
        )
    if not what_is_not_working:
        what_is_not_working.append("No strong underperformance pattern stands out yet beyond the normal drop-off through the funnel.")

    recommended_search_adjustments: list[str] = []
    if negative_role and negative_role.net_score < 0:
        recommended_search_adjustments.append(
            f"Reduce search volume around {negative_role.key} patterns because they are drawing repeated hide or bad-fit feedback."
        )
    if analytics.top_matched_roles:
        recommended_search_adjustments.append(
            f"Keep titles and terms closer to {', '.join(analytics.top_matched_roles[:2])} because those are the strongest deterministic role matches."
        )
    if analytics.top_matched_skills:
        recommended_search_adjustments.append(
            f"Keep search terms anchored to {', '.join(analytics.top_matched_skills[:3])} instead of broadening into weaker skill clusters."
        )
    if behavior.search_run_count > 0 and funnel.save_count == 0:
        recommended_search_adjustments.append(
            f"{behavior.search_run_count} search runs have not produced saves yet, so narrow the query set before adding more volume."
        )

    recommended_source_moves: list[str] = []
    if positive_source and positive_source.net_score > 0:
        recommended_source_moves.append(
            f"Prioritize {positive_source.key} first because it has the strongest positive interaction signal."
        )
    if negative_source and negative_source.net_score < 0:
        recommended_source_moves.append(
            f"Reduce time spent on {negative_source.key} until it shows better save or application outcomes."
        )
    if not positive_source and analytics.jobs_by_source:
        recommended_source_moves.append(
            f"Start by testing the largest available source, {analytics.jobs_by_source[0].source}, and compare its save rate against the others."
        )
    if funnel.applications_by_source:
        best_application_source = funnel.applications_by_source[0]
        recommended_source_moves.append(
            f"Use {best_application_source.source} as the reference source for conversion quality because it currently leads in applications."
        )
    elif funnel.saves_by_source:
        best_save_source = funnel.saves_by_source[0]
        recommended_source_moves.append(
            f"Review why {best_save_source.source} is producing the most saves and reuse that source pattern next week."
        )

    recommended_role_focus: list[str] = []
    if llm_context.analyzed_profile:
        recommended_role_focus.append(
            f"Stay centered on {primary_role} because it is still the clearest role anchor in the deterministic profile."
        )
    if positive_role and positive_role.net_score > 0:
        recommended_role_focus.append(
            f"Keep prioritizing {positive_role.key} opportunities because that role family has the strongest positive behavior signal."
        )
    for role in analytics.top_matched_roles[:2]:
        label = role.replace("_", " ")
        if label.casefold() not in {item.casefold() for item in recommended_role_focus}:
            recommended_role_focus.append(
                f"Use {label} as a secondary focus only when the role still overlaps with the matched skills and keywords."
            )
    if negative_role and negative_role.net_score < 0:
        recommended_role_focus.append(
            f"De-prioritize {negative_role.key} work for now because the current behavior summary is more negative than positive."
        )

    funnel_bottlenecks: list[str] = []
    if funnel.impression_count > 0 and funnel.open_count == 0:
        funnel_bottlenecks.append("Impressions are not turning into opens, so titles and source quality need review.")
    elif funnel.conversion_rates.open_rate_from_impressions < 0.25 and funnel.impression_count > 0:
        funnel_bottlenecks.append(
            f"Open rate from impressions is {round(funnel.conversion_rates.open_rate_from_impressions * 100)}%, which suggests weak first-click relevance."
        )
    if funnel.open_count > 0 and funnel.conversion_rates.save_rate_from_opens < 0.2:
        funnel_bottlenecks.append(
            f"Save rate from opens is {round(funnel.conversion_rates.save_rate_from_opens * 100)}%, so many opened jobs are not convincing enough to keep."
        )
    if funnel.save_count > 0 and funnel.conversion_rates.application_rate_from_saves < 0.35:
        funnel_bottlenecks.append(
            f"Application rate from saves is {round(funnel.conversion_rates.application_rate_from_saves * 100)}%, so saved jobs are not converting efficiently."
        )
    if funnel.bad_fit_count > 0:
        funnel_bottlenecks.append(
            f"{funnel.bad_fit_count} bad-fit events indicate that at least part of the search set is still misaligned."
        )
    if not funnel_bottlenecks:
        funnel_bottlenecks.append("No severe funnel bottleneck is visible yet; keep monitoring the next save-to-application step.")

    next_week_plan: list[str] = []
    if positive_source and positive_source.net_score > 0:
        next_week_plan.append(
            f"Run the first search sessions on {positive_source.key} and keep the terms close to {primary_role} work."
        )
    else:
        next_week_plan.append(f"Run a small number of focused searches around {primary_role} instead of expanding broadly.")
    next_week_plan.append(
        "Review saved jobs at the end of each search session and remove search terms that repeatedly lead to hides or bad fits."
    )
    if funnel.save_count > funnel.application_created_count:
        next_week_plan.append(
            "Turn the strongest saved jobs into applications faster so the funnel does not stall at the save stage."
        )
    if negative_source and negative_source.net_score < 0:
        next_week_plan.append(
            f"Limit experimental time on {negative_source.key} until the negative signal weakens."
        )

    weekly_summary_parts: list[str] = []
    weekly_summary_parts.append(
        f"This week the clearest grounded pattern is around {primary_role} searches and a funnel of {funnel.open_count} opens, {funnel.save_count} saves, and {funnel.application_created_count} applications."
    )
    if positive_source and positive_source.net_score > 0:
        weekly_summary_parts.append(
            f"{positive_source.key} is the strongest positive source signal in the behavior summary."
        )
    if negative_source and negative_source.net_score < 0:
        weekly_summary_parts.append(
            f"{negative_source.key} is the main negative source pattern to watch."
        )
    if not positive_source and not negative_source:
        weekly_summary_parts.append(
            "Source-level evidence is still limited, so recommendations should stay conservative."
        )

    return {
        "weekly_summary": " ".join(weekly_summary_parts).strip(),
        "what_is_working": what_is_working,
        "what_is_not_working": what_is_not_working,
        "recommended_search_adjustments": recommended_search_adjustments,
        "recommended_source_moves": recommended_source_moves,
        "recommended_role_focus": recommended_role_focus,
        "funnel_bottlenecks": funnel_bottlenecks,
        "next_week_plan": next_week_plan,
    }
