package script

import "strings"

type RuntimeConfig struct {
	StepLimit int
}

func DefaultRuntimeConfig() RuntimeConfig {
	return RuntimeConfig{
		StepLimit: 10_000,
	}
}

type HostBindings interface {
	Call(method string, args []Value) (Value, error)
}

type DispatchRequest struct {
	Source string
}

type DispatchResult struct {
	Value Value
}

type Runtime struct {
	config RuntimeConfig
	host   HostBindings
}

func NewRuntime(host HostBindings) *Runtime {
	return NewRuntimeWithConfig(DefaultRuntimeConfig(), host)
}

func NewRuntimeWithConfig(config RuntimeConfig, host HostBindings) *Runtime {
	cfg := config
	if cfg.StepLimit <= 0 {
		cfg.StepLimit = DefaultRuntimeConfig().StepLimit
	}
	return &Runtime{
		config: cfg,
		host:   host,
	}
}

func (r *Runtime) Config() RuntimeConfig {
	if r == nil {
		return DefaultRuntimeConfig()
	}
	return r.config
}

func (r *Runtime) Dispatch(request DispatchRequest) (DispatchResult, error) {
	if r == nil {
		return DispatchResult{}, NewError(ErrorKindRuntime, "script runtime is unavailable")
	}

	source := strings.TrimSpace(request.Source)
	if source == "" || source == "noop" {
		return DispatchResult{Value: UndefinedValue()}, nil
	}

	if strings.HasPrefix(source, "host:") {
		method := strings.TrimSpace(strings.TrimPrefix(source, "host:"))
		if method == "" {
			return DispatchResult{}, NewError(ErrorKindParse, "host dispatch requires a non-empty method name")
		}
		if r.host == nil {
			return DispatchResult{}, NewError(ErrorKindHost, "host bindings are unavailable")
		}
		value, err := r.host.Call(method, nil)
		if err != nil {
			return DispatchResult{}, NewError(ErrorKindHost, err.Error())
		}
		return DispatchResult{Value: value}, nil
	}

	return DispatchResult{}, NewError(
		ErrorKindUnsupported,
		"unsupported script source; this scaffold supports only `noop` and `host:<method>`",
	)
}
