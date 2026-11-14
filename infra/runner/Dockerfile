# IGNIS analysis runner image (`analysis_execute`).
#
# The host pins this image by its content address (image id) and refuses any other; rebuild with
# tools/runner/build.sh and update IGNIS_RUNNER_IMAGE. Nothing here is trusted by the host: the
# isolation comes from how the container is started (no network, read-only root, uid 65534, no
# capabilities, explicit mounts) and, in deployment, from gVisor (`--runtime runsc`).
#
# Base pinned by digest: python:3.12-slim-bookworm.
FROM python@sha256:935c3c5b12357a7e10caa9c1e6ca5692344d97cf229832ef584be2cec9fc03df

COPY requirements.lock /opt/ignis/requirements.lock
RUN python -m pip install --no-cache-dir --no-compile --require-hashes --only-binary :all: \
        -r /opt/ignis/requirements.lock \
    && python -m pip uninstall -y pip setuptools wheel 2>/dev/null; \
    python -c "import numpy, scipy, pandas, pyarrow, sympy, pint" \
    && mkdir -p /in /out /job \
    && rm -rf /root/.cache

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    HOME=/tmp \
    MPLCONFIGDIR=/tmp

LABEL org.ignis.runner="analysis" \
      org.ignis.runner.contract="ExecutionRecord"

USER 65534:65534
WORKDIR /out
