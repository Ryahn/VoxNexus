import mongoose from 'mongoose';
import utility from '../../utils';

const RoleSchema = new mongoose.Schema({
  _id: {
    type: String,
    default: () => utility.generateSnowflake().toString()
  },
  name: {
    type: String,
    required: true,
    maxlength: 100,
    trim: true,
    validate: {
      validator: function(v) {
        return /^[a-zA-Z0-9\s\-_]+$/.test(v);
      },
      message: 'Role name can only contain letters, numbers, spaces, hyphens, and underscores'
    }
  },
  color: {
    type: String,
    default: '#000000'
  },
  permissions: {
    type: [String],
    default: []
  },
  server: {
    type: String,
    ref: 'Server',
    required: true
  },
  createdAt: {
    type: Date,
    default: Date.now
  },
  updatedAt: {
    type: Date,
    default: Date.now
  }
}, {
  timestamps: true
});

RoleSchema.pre('save', function(next) {
  this.updatedAt = new Date();
  next();
});

RoleSchema.index({ server: 1, name: 1 }, { unique: true });
RoleSchema.index({ color: 1, permissions: 1 });

// Method to add permission to role
RoleSchema.methods.addPermission = async function(permission) {
  if (!this.permissions.includes(permission)) {
    this.permissions.push(permission);
    await this.save();
  }
};

// Method to remove permission from role
RoleSchema.methods.removePermission = async function(permission) {
  this.permissions = this.permissions.filter(p => p !== permission);
  await this.save();
};

// Method to check if role has permission
RoleSchema.methods.hasPermission = function(permission) {
  return this.permissions.includes(permission);
};

// Method to update role color
RoleSchema.methods.updateColor = async function(color) {
  if (/^#([0-9a-fA-F]{6})$/.test(color)) {
    this.color = color;
    await this.save();
  }
};

// Static method to get all roles for a server
RoleSchema.statics.getServerRoles = async function(serverId) {
  return this.find({ server: serverId })
    .sort({ createdAt: 1 });
};

// Static method to get roles with specific permission
RoleSchema.statics.getRolesWithPermission = async function(serverId, permission) {
  return this.find({
    server: serverId,
    permissions: permission
  });
};

// Static method to delete all roles for a server
RoleSchema.statics.deleteAllForServer = async function(serverId) {
  return this.deleteMany({ server: serverId });
};

// Static method to update permissions for multiple roles
RoleSchema.statics.updatePermissions = async function(roleIds, permissions) {
  return this.updateMany(
    { _id: { $in: roleIds } },
    { $set: { permissions } }
  );
};

export default mongoose.model('Role', RoleSchema);