"""Describe per-class dispersion; aggregate acceptance does not certify each class."""
from pathlib import Path
from statistics import median
import json

out = Path('/Users/francescotinti/Claude/phpr-s179-cost-resume-results')
runs = json.loads((out / 'dense-analysis.json').read_text())['sampled_runs']
result = {}
for cls in sorted(set().union(*(r['classes'] for r in runs))):
    row = {}
    for rate in (16, 64):
        selected = [r for r in runs if r['rate'] == rate]
        values = [r['classes'].get(cls, {}).get('estimated_ns', 0) / 1e6 for r in selected]
        samples = sum(r['classes'].get(cls, {}).get('n', 0) for r in selected)
        row[str(rate)] = dict(median_ms=median(values), min_ms=min(values),
                              max_ms=max(values), samples=samples,
                              underresolved=samples < 200)
    denominator = row['64']['median_ms']
    row['median_gap'] = abs(row['16']['median_ms'] / denominator - 1) if denominator else None
    result[cls] = row
(out / 'class-summary.json').write_text(json.dumps(result, indent=2))
