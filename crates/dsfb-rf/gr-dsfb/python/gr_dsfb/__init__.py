"""gr_dsfb — GNU Radio Python package for DSFB-RF structural anomaly detection.

Phase I OOT module: read-only tap for USRP B200 / X310 / LimeSDR / RTL-SDR.
"""

from .dsfb_sink_b200 import dsfb_sink_b200

__all__ = ["dsfb_sink_b200"]
__version__ = "1.0.0"
