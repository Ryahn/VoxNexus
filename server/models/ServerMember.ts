import mongoose, { Schema, Document, Model, Types } from 'mongoose';

export interface IServerMember extends Document {
  serverId: Types.ObjectId;
  userId: Types.ObjectId;
  roleIds: Types.ObjectId[];
}

const ServerMemberSchema: Schema<IServerMember> = new Schema({
  serverId: { type: Schema.Types.ObjectId, ref: 'Server', required: true },
  userId: { type: Schema.Types.ObjectId, ref: 'User', required: true },
  roleIds: [{ type: Schema.Types.ObjectId, ref: 'Role', required: true }],
});

export default (mongoose.models.ServerMember as Model<IServerMember>) || mongoose.model<IServerMember>('ServerMember', ServerMemberSchema); 