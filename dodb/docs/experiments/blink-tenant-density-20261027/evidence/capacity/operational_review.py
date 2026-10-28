from capacity_review import reduce_report as baseline_reduce_report


def reduce_report(report, source_commit, worker_log):
    pressure = report['backend_memory_pressure']
    final = pressure['cell']['final']
    shutdown_deadline = (report['status'] == 'failed_or_inconclusive'
                         and final['memory.events']['oom_kill'] == 0
                         and 'diagnostic worker shutdown exceeded 15 seconds' in worker_log
                         and 'diagnostic worker exited unsuccessfully' in report.get('failure_reason', ''))
    if not shutdown_deadline:
        return baseline_reduce_report(report, source_commit)
    configuration = report['effective_configuration']
    assert pressure['enabled'] and not pressure['samples_truncated']
    assert final['cgroup.events']['populated'] == 0
    assert pressure['cell']['event_delta']['high'] > 0
    for snapshot in [final, *[epoch['final'] for epoch in pressure['epochs'].values()]]:
        assert snapshot['memory.max'] == 256 * 1024**2
        assert snapshot['memory.high'] == 192 * 1024**2
        assert snapshot['memory.swap.max'] == snapshot['memory.swap.current'] == 0
        assert snapshot['cgroup.procs'] == []
    diagnostic = report['memory_diagnostic']
    assert diagnostic['source_revision'] == source_commit
    assert all(event['source_revision'] == source_commit for event in diagnostic['worker_lifecycle'] if 'source_revision' in event)
    assert pressure['epochs']['setup']['final']['memory.peak'] > 0
    assert all(pressure['epochs'][name]['final']['memory.peak'] == 0 for name in ['measurement', 'oracle', 'reopen-oracle'])
    return {
        'backend': configuration['backend'], 'registered': configuration['registered_tenants'],
        'active': configuration['active_tenants'], 'initial_rows': configuration['initial_rows_per_tenant'],
        'duration_ms': configuration['duration_ms'],
        'sample_interval_ms': configuration['backend_memory_budget']['sample_interval_ms'],
        'status': report['status'], 'cell_peak_bytes': final['memory.peak'],
        'cell_events': pressure['cell']['event_delta'],
        'epoch_peaks_bytes': {name: epoch['final']['memory.peak'] for name, epoch in pressure['epochs'].items()},
        'epoch_events': {name: epoch['local_event_delta'] for name, epoch in pressure['epochs'].items()},
        'operational_pass': False, 'slo_pass': False, 'resource_failure': False,
        'operational_failure': True, 'operational_timeout': True,
        'failure_kind': 'setup_worker_shutdown_deadline', 'failure_epochs': ['setup'],
        'failure_reason': report['failure_reason'], 'worker_shutdown_deadline_seconds': 15,
        'offered_load_started': False,
        'scope': 'Setup shutdown deadline exceeded with high events; no OOM, measured throughput, or complete-population oracle claim',
    }
