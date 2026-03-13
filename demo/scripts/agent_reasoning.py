#!/usr/bin/env python3
"""
LLM-powered agent reasoning for offer comparison.

Uses a reasoning model (OpenAI o3-mini by default) to analyze competing
provider offers against a user's natural-language objective — the step
that makes the agent an actual *agent* rather than a scoring function.

Falls back to deterministic scoring if no API key is configured.
"""

import json
import os
from typing import Any, Dict, List, Optional


SUPPORTED_PROVIDERS = {"openai", "anthropic"}


def _get_llm_config() -> Optional[Dict[str, str]]:
    provider = os.environ.get("DEMO_LLM_PROVIDER", "openai").lower()
    if provider not in SUPPORTED_PROVIDERS:
        return None

    if provider == "openai":
        api_key = os.environ.get("OPENAI_API_KEY")
        if not api_key:
            return None
        return {
            "provider": "openai",
            "api_key": api_key,
            "model": os.environ.get("DEMO_LLM_MODEL", "o3-mini"),
        }

    if provider == "anthropic":
        api_key = os.environ.get("ANTHROPIC_API_KEY")
        if not api_key:
            return None
        return {
            "provider": "anthropic",
            "api_key": api_key,
            "model": os.environ.get("DEMO_LLM_MODEL", "claude-sonnet-4-20250514"),
        }

    return None


def _build_system_prompt() -> str:
    return (
        "You are an AI agent operating within the Covenant Protocol. "
        "Your role is to compare provider offers against a user's objective "
        "and recommend the best option.\n\n"
        "You are NOT executing the booking. You are selecting the best commitment "
        "from competing providers who will each stand behind their offer terms.\n\n"
        "Your job:\n"
        "1. Verify each offer satisfies the hard constraints (budget, stops, layover).\n"
        "2. Analyze tradeoffs between qualifying offers across price, convenience, "
        "cabin class, refund flexibility, and hold window.\n"
        "3. Weigh the user's stated preferences.\n"
        "4. Reason step by step about which offer best serves the user's intent.\n"
        "5. Produce a final ranked recommendation with clear justification.\n\n"
        "Be direct. Show your reasoning. Name tradeoffs explicitly."
    )


def _build_comparison_prompt(
    objective: Dict[str, Any],
    offers: List[Dict[str, Any]],
) -> str:
    offers_block = json.dumps(offers, indent=2)
    return (
        f"## User Objective\n\n"
        f"Route: {objective['route']['origin']} → {objective['route']['destination']} "
        f"on {objective['route']['departure_date']}\n\n"
        f"Hard constraints:\n"
        f"- Max budget: ${objective['constraints']['max_budget_usd']}\n"
        f"- Max stops: {objective['constraints']['max_stops']}\n"
        f"- No overnight layover: {objective['constraints']['no_overnight_layover']}\n"
        f"- Cabin: {objective['constraints']['cabin_class']}\n\n"
        f"Preferences:\n"
        f"- Prefer nonstop: {objective['preferences']['prefer_nonstop']}\n"
        f"- Price weight: {objective['preferences']['weight_price']}\n"
        f"- Convenience weight: {objective['preferences']['weight_convenience']}\n"
        f"- Refund flexibility weight: {objective['preferences']['weight_refund_flexibility']}\n\n"
        f"## Provider Offers\n\n"
        f"```json\n{offers_block}\n```\n\n"
        f"## Instructions\n\n"
        f"1. Check each offer against hard constraints. Disqualify any that violate them.\n"
        f"2. For qualifying offers, reason through the tradeoffs step by step.\n"
        f"3. Produce your final ranking.\n\n"
        f"Respond with the following JSON structure (no markdown fencing):\n"
        f'{{\n'
        f'  "reasoning": "<your step-by-step analysis>",\n'
        f'  "ranked_offers": [\n'
        f'    {{\n'
        f'      "offer_id": "<id>",\n'
        f'      "rank": 1,\n'
        f'      "recommendation": "<one-line summary of why this ranks here>"\n'
        f'    }}\n'
        f'  ],\n'
        f'  "selected_offer_id": "<offer_id of your top recommendation>",\n'
        f'  "confidence": "<high|medium|low>"\n'
        f'}}'
    )


def _call_openai(config: Dict[str, str], system: str, user: str) -> str:
    import urllib.request

    body = json.dumps({
        "model": config["model"],
        "messages": [
            {"role": "developer", "content": system},
            {"role": "user", "content": user},
        ],
    }).encode("utf-8")

    req = urllib.request.Request(
        "https://api.openai.com/v1/chat/completions",
        data=body,
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {config['api_key']}",
        },
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=120) as resp:
        data = json.loads(resp.read().decode("utf-8"))

    return data["choices"][0]["message"]["content"]


def _call_anthropic(config: Dict[str, str], system: str, user: str) -> str:
    import urllib.request

    body = json.dumps({
        "model": config["model"],
        "max_tokens": 4096,
        "system": system,
        "messages": [
            {"role": "user", "content": user},
        ],
    }).encode("utf-8")

    req = urllib.request.Request(
        "https://api.anthropic.com/v1/messages",
        data=body,
        headers={
            "Content-Type": "application/json",
            "x-api-key": config["api_key"],
            "anthropic-version": "2023-06-01",
        },
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=120) as resp:
        data = json.loads(resp.read().decode("utf-8"))

    return data["content"][0]["text"]


def _parse_llm_response(raw: str) -> Dict[str, Any]:
    text = raw.strip()
    if text.startswith("```"):
        text = text.split("\n", 1)[1] if "\n" in text else text[3:]
        if text.endswith("```"):
            text = text[:-3]
        text = text.strip()

    return json.loads(text)


def reason_over_offers(
    objective: Dict[str, Any],
    offers: List[Dict[str, Any]],
) -> Optional[Dict[str, Any]]:
    """
    Call a reasoning model to compare offers against the objective.

    Returns a dict with keys: reasoning, ranked_offers, selected_offer_id, confidence.
    Returns None if no LLM is configured (caller should fall back to deterministic scoring).
    """
    config = _get_llm_config()
    if config is None:
        return None

    system_prompt = _build_system_prompt()
    user_prompt = _build_comparison_prompt(objective, offers)

    print(f"[agent] Calling reasoning model: {config['provider']}/{config['model']}")

    if config["provider"] == "openai":
        raw = _call_openai(config, system_prompt, user_prompt)
    elif config["provider"] == "anthropic":
        raw = _call_anthropic(config, system_prompt, user_prompt)
    else:
        return None

    print(f"[agent] Reasoning model responded ({len(raw)} chars)")

    try:
        result = _parse_llm_response(raw)
    except (json.JSONDecodeError, KeyError, IndexError):
        print(f"[agent] WARNING: Could not parse structured response, returning raw reasoning")
        result = {
            "reasoning": raw,
            "ranked_offers": [],
            "selected_offer_id": None,
            "confidence": "low",
            "parse_error": True,
        }

    result["model"] = f"{config['provider']}/{config['model']}"
    result["raw_response"] = raw
    return result
