import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { Package, Handshake, TrendingUp, Shield, Plus, ArrowRight } from 'lucide-react';
import { materialApi } from '../../api/material';
import { handshakeApi } from '../../api/handshake';
import apiClient from '../../lib/api';

interface DashboardStats {
  totalMaterials: number;
  activeHandshakes: number;
  icsScore: string;
  complianceStatus: string;
}

export function DashboardPage() {
  const [stats, setStats] = useState<DashboardStats>({
    totalMaterials: 0,
    activeHandshakes: 0,
    icsScore: '--',
    complianceStatus: 'Loading...',
  });
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadDashboardData();
  }, []);

  const loadDashboardData = async () => {
    try {
      const [materialSummary, handshakeList] = await Promise.all([
        materialApi.getMaterialSummary().catch(() => ({ total_materials: 0 })),
        handshakeApi.listHandshakes().catch(() => []),
      ]);

      // Fetch score if available
      let icsScore = '--';
      try {
        const token = localStorage.getItem('auth_token');
        const user = JSON.parse(localStorage.getItem('user') || '{}');
        if (user?.id) {
          const scoreResponse = await apiClient.get(`/scores/${user.id}`, {
            headers: { Authorization: `Bearer ${token}` },
          });
          if (scoreResponse.data.success && scoreResponse.data.data?.ics_score) {
            icsScore = scoreResponse.data.data.ics_score.toString();
          }
        }
      } catch (e) {
        // Score not available yet
      }

      setStats({
        totalMaterials: materialSummary.total_materials || 0,
        activeHandshakes: Array.isArray(handshakeList) ? handshakeList.length : 0,
        icsScore,
        complianceStatus: 'Good',
      });
    } catch (error) {
      console.error('Failed to load dashboard data:', error);
    } finally {
      setLoading(false);
    }
  };

  const displayStats = [
    { name: 'Total Materials', value: stats.totalMaterials.toString(), icon: Package, change: '+0%', href: '/materials' },
    { name: 'Active Handshakes', value: stats.activeHandshakes.toString(), icon: Handshake, change: '+0%', href: '/handshakes' },
    { name: 'ICS Score', value: stats.icsScore, icon: TrendingUp, change: 'N/A', href: '/scores' },
    { name: 'Compliance Status', value: stats.complianceStatus, icon: Shield, change: 'On track', href: '/compliance' },
  ];

  if (loading) {
    return (
      <div className="flex items-center justify-center p-12">
        <div className="text-lg text-gray-500">Loading dashboard...</div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Dashboard</h1>
          <p className="mt-1 text-sm text-gray-500">
            Welcome to B-Trace Protocol - Track your materials and manage credit
          </p>
        </div>
        <Link
          to="/materials/new"
          className="inline-flex items-center px-4 py-2 border border-transparent rounded-lg shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
        >
          <Plus className="h-5 w-5 mr-2" />
          New Material
        </Link>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
        {stats.map((stat) => (
          <Link
            key={stat.name}
            to={stat.href}
            className="relative overflow-hidden bg-white rounded-xl shadow-sm hover:shadow-md transition-shadow p-6 border border-gray-200"
          >
            <dt>
              <div className="absolute rounded-md bg-indigo-500 p-3">
                <stat.icon className="h-6 w-6 text-white" aria-hidden="true" />
              </div>
              <p className="ml-16 truncate text-sm font-medium text-gray-500">{stat.name}</p>
            </dt>
            <dd className="ml-16 flex items-baseline">
              <p className="text-2xl font-semibold text-gray-900">{stat.value}</p>
              <p className={`ml-2 flex items-baseline text-sm font-semibold ${
                stat.change.includes('+') ? 'text-green-600' : 
                stat.change === 'N/A' ? 'text-gray-400' : 'text-gray-600'
              }`}>
                {stat.change}
              </p>
            </dd>
          </Link>
        ))}
      </div>

      {/* Quick Actions */}
      <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">Quick Actions</h2>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          <Link
            to="/materials/new"
            className="flex items-center p-4 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors group"
          >
            <div className="flex-shrink-0 h-10 w-10 bg-indigo-100 rounded-lg flex items-center justify-center">
              <Package className="h-6 w-6 text-indigo-600" />
            </div>
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-900">Create Material Batch</p>
              <p className="text-xs text-gray-500">Log new industrial material</p>
            </div>
            <ArrowRight className="ml-auto h-5 w-5 text-gray-400 group-hover:text-gray-600" />
          </Link>

          <Link
            to="/handshakes/initiate"
            className="flex items-center p-4 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors group"
          >
            <div className="flex-shrink-0 h-10 w-10 bg-green-100 rounded-lg flex items-center justify-center">
              <Handshake className="h-6 w-6 text-green-600" />
            </div>
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-900">Initiate Handshake</p>
              <p className="text-xs text-gray-500">Transfer material ownership</p>
            </div>
            <ArrowRight className="ml-auto h-5 w-5 text-gray-400 group-hover:text-gray-600" />
          </Link>

          <Link
            to="/scores"
            className="flex items-center p-4 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors group"
          >
            <div className="flex-shrink-0 h-10 w-10 bg-purple-100 rounded-lg flex items-center justify-center">
              <TrendingUp className="h-6 w-6 text-purple-600" />
            </div>
            <div className="ml-4">
              <p className="text-sm font-medium text-gray-900">View Credit Score</p>
              <p className="text-xs text-gray-500">Check ICS score & limits</p>
            </div>
            <ArrowRight className="ml-auto h-5 w-5 text-gray-400 group-hover:text-gray-600" />
          </Link>
        </div>
      </div>

      {/* Recent Activity Placeholder */}
      <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">Recent Activity</h2>
        <div className="text-center py-12">
          <Package className="mx-auto h-12 w-12 text-gray-400" />
          <p className="mt-4 text-sm text-gray-500">No recent activity</p>
          <p className="mt-1 text-xs text-gray-400">Start by creating your first material batch</p>
        </div>
      </div>

      {/* Getting Started Guide */}
      <div className="bg-gradient-to-r from-indigo-500 to-purple-600 rounded-xl shadow-sm p-6 text-white">
        <h2 className="text-lg font-semibold mb-2">Getting Started with B-Trace</h2>
        <ol className="mt-4 space-y-3 text-sm">
          <li className="flex items-start">
            <span className="flex-shrink-0 h-6 w-6 bg-white/20 rounded-full flex items-center justify-center text-xs font-medium">1</span>
            <span className="ml-3">Create a material batch with weight, grade, and source pincode</span>
          </li>
          <li className="flex items-start">
            <span className="flex-shrink-0 h-6 w-6 bg-white/20 rounded-full flex items-center justify-center text-xs font-medium">2</span>
            <span className="ml-3">Initiate a handshake to transfer ownership to a buyer</span>
          </li>
          <li className="flex items-start">
            <span className="flex-shrink-0 h-6 w-6 bg-white/20 rounded-full flex items-center justify-center text-xs font-medium">3</span>
            <span className="ml-3">Both parties confirm via QR code scan and digital signature</span>
          </li>
          <li className="flex items-start">
            <span className="flex-shrink-0 h-6 w-6 bg-white/20 rounded-full flex items-center justify-center text-xs font-medium">4</span>
            <span className="ml-3">Build your credit history and access better financing rates</span>
          </li>
        </ol>
      </div>
    </div>
  );
}
