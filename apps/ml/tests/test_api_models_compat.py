from app.api_models import (
    BootstrapRequest as CanonicalBootstrapRequest,
)
from app.api_models import (
    FitAnalyzeRequest as CanonicalFitAnalyzeRequest,
)
from app.models import BootstrapRequest as LegacyBootstrapRequest
from app.scoring_models import FitAnalyzeRequest as LegacyFitAnalyzeRequest


def test_legacy_model_modules_reexport_canonical_api_models():
    assert LegacyFitAnalyzeRequest is CanonicalFitAnalyzeRequest
    assert LegacyBootstrapRequest is CanonicalBootstrapRequest
