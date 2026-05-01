import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useAuth } from '../auth/AuthProvider';

interface ScoreMetrics {
  icsScore: number;
  pdScore: number;
  stabilityScore: number;
  creditLimit: number;
  riskCategory: 'Low' | 'Medium' | 'High';
}

export function ScoreDashboard() {
  const { user } = useAuth();
  
  // Mock data - will be fetched from API
  const metrics: ScoreMetrics = {
    icsScore: 725,
    pdScore: 8.5,
    stabilityScore: 72,
    creditLimit: 5000000,
    riskCategory: 'Medium',
  };

  const getScoreColor = (score: number) => {
    if (score >= 750) return 'text-green-600';
    if (score >= 650) return 'text-yellow-600';
    return 'text-red-600';
  };

  const getRiskBadgeColor = (risk: string) => {
    switch (risk) {
      case 'Low': return 'bg-green-100 text-green-800';
      case 'Medium': return 'bg-yellow-100 text-yellow-800';
      case 'High': return 'bg-red-100 text-red-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Credit Score Dashboard</h1>
        <p className="text-muted-foreground">
          View your Industrial Credit Score and financial health metrics
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">ICS Score</CardTitle>
          </CardHeader>
          <CardContent>
            <div className={`text-3xl font-bold ${getScoreColor(metrics.icsScore)}`}>
              {metrics.icsScore}
            </div>
            <p className="text-xs text-muted-foreground">
              {metrics.icsScore >= 750 ? 'Excellent' : metrics.icsScore >= 650 ? 'Good' : 'Needs Improvement'}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Probability of Default</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold">{metrics.pdScore}%</div>
            <p className="text-xs text-muted-foreground">
              Lower is better
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Stability Score</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold">{metrics.stabilityScore}/100</div>
            <p className="text-xs text-muted-foreground">
              Based on transaction history
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Credit Limit</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold">
              ₹{(metrics.creditLimit / 100000).toFixed(1)}L
            </div>
            <p className="text-xs text-muted-foreground">
              Available for trade financing
            </p>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Risk Assessment</CardTitle>
            <CardDescription>Current risk category based on scoring model</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex items-center justify-between">
              <span className="text-lg font-medium">Risk Category</span>
              <span className={`px-3 py-1 rounded-full text-sm font-medium ${getRiskBadgeColor(metrics.riskCategory)}`}>
                {metrics.riskCategory}
              </span>
            </div>
            <div className="mt-4 space-y-2">
              <div className="flex justify-between text-sm">
                <span>Transaction Volume</span>
                <span className="font-medium">High</span>
              </div>
              <div className="flex justify-between text-sm">
                <span>Payment History</span>
                <span className="font-medium">Good</span>
              </div>
              <div className="flex justify-between text-sm">
                <span>Handshake Confirmations</span>
                <span className="font-medium">95%</span>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Score Factors</CardTitle>
            <CardDescription>Key factors affecting your score</CardDescription>
          </CardHeader>
          <CardContent>
            <ul className="space-y-3">
              <li className="flex items-start gap-2">
                <svg className="w-5 h-5 text-green-500 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
                <span className="text-sm">Consistent material logging over 6 months</span>
              </li>
              <li className="flex items-start gap-2">
                <svg className="w-5 h-5 text-green-500 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
                <span className="text-sm">High handshake confirmation rate (95%)</span>
              </li>
              <li className="flex items-start gap-2">
                <svg className="w-5 h-5 text-yellow-500 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
                <span className="text-sm">Limited diversity in trading partners</span>
              </li>
              <li className="flex items-start gap-2">
                <svg className="w-5 h-5 text-yellow-500 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
                <span className="text-sm">Recent increase in transaction volume</span>
              </li>
            </ul>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
