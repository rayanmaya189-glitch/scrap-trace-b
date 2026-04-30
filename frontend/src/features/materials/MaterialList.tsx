import React from 'react';
import { Package, AlertCircle } from 'lucide-react';

export function MaterialList() {
  // Empty state - no materials yet
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Materials</h1>
          <p className="mt-1 text-sm text-gray-500">
            Track and manage your industrial material batches
          </p>
        </div>
      </div>

      <div className="bg-white rounded-xl shadow-sm border border-gray-200">
        <div className="text-center py-16">
          <Package className="mx-auto h-16 w-16 text-gray-300" />
          <h3 className="mt-4 text-lg font-medium text-gray-900">No materials yet</h3>
          <p className="mt-2 text-sm text-gray-500 max-w-sm mx-auto">
            Get started by creating your first material batch. You'll need the weight, grade, and source location.
          </p>
          <div className="mt-6">
            <a
              href="/materials/new"
              className="inline-flex items-center px-4 py-2 border border-transparent rounded-lg shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
            >
              <Package className="h-5 w-5 mr-2" />
              Create Material
            </a>
          </div>
        </div>
      </div>

      {/* Info cards */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <div className="flex">
            <AlertCircle className="h-5 w-5 text-blue-400" />
            <div className="ml-3">
              <h3 className="text-sm font-medium text-blue-800">What is a Material Passport?</h3>
              <p className="mt-1 text-sm text-blue-700">
                A digital record of your industrial material batch containing weight, grade, origin, and ownership history. Each batch gets a unique QR code for tracking.
              </p>
            </div>
          </div>
        </div>

        <div className="bg-green-50 border border-green-200 rounded-lg p-4">
          <div className="flex">
            <AlertCircle className="h-5 w-5 text-green-400" />
            <div className="ml-3">
              <h3 className="text-sm font-medium text-green-800">Why Log Materials?</h3>
              <p className="mt-1 text-sm text-green-700">
                Every verified transaction builds your credit history. Higher quality records lead to better ICS scores and lower financing rates.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
