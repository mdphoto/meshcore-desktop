import { Component } from 'react'
import type { ReactNode, ErrorInfo } from 'react'
import { AlertCircle, RefreshCw } from 'lucide-react'

interface Props {
  children: ReactNode
}

interface State {
  hasError: boolean
  error: Error | null
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props)
    this.state = { hasError: false, error: null }
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('ErrorBoundary caught:', error, info)
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="flex h-full flex-col items-center justify-center gap-4 p-8 text-gray-400">
          <AlertCircle className="h-12 w-12 text-red-400" />
          <h2 className="text-lg font-semibold text-white">Une erreur est survenue</h2>
          <p className="max-w-md text-center text-sm">
            {this.state.error?.message ?? 'Erreur inconnue'}
          </p>
          <button
            onClick={() => this.setState({ hasError: false, error: null })}
            className="flex items-center gap-2 rounded-lg bg-gray-800 px-4 py-2 text-sm text-gray-300 hover:bg-gray-700 transition-colors"
          >
            <RefreshCw className="h-4 w-4" />
            Réessayer
          </button>
        </div>
      )
    }

    return this.props.children
  }
}
