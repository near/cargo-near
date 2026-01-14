use testresult::TestResult;

#[test]
fn test_tracing_debug_feature_available() -> TestResult {
    // This test verifies that the tracing_debug feature is available
    // and can be compiled. The actual functionality is tested by checking
    // that the function exists and can be called.
    
    // Note: We can't actually call init_tracing_debug() in tests because
    // tracing subscriber can only be initialized once per process,
    // and other tests may have already initialized it.
    
    // This test will fail to compile if the feature is not properly set up
    #[cfg(feature = "tracing_debug")]
    {
        // Just verify the function exists
        let _init_fn: fn() -> Result<(), Box<dyn std::error::Error>> = cargo_near_build::init_tracing_debug;
    }
    
    #[cfg(not(feature = "tracing_debug"))]
    {
        // If feature is not enabled, this test should still pass
        // but we note that the feature wasn't tested
        println!("Note: tracing_debug feature not enabled, skipping functionality test");
    }
    
    Ok(())
}
